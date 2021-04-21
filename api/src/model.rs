use rocket::{http::{Status, ContentType}, Request, Data};
use rocket::data::{Outcome, FromData, ToByteUnit};
use rocket_contrib::databases::postgres;
use serde::{Serialize, Deserialize};
use serde_json::{from_str, Value};
use crate::errors::ErrorMessage;
use serde_json::ser::to_string;


// Limit is 1MB here, should be enough for common use. If you are sending
// anything bigger, you should be required to provide appropriate header.
const DEFAULT_BUFFER_LIMIT: u32 = 1024 * 1024;


#[database("storage")]
pub struct ApiDatabase(postgres::Client);


#[derive(Deserialize, Clone, Debug)]
pub struct Entry(pub Value);


#[derive(Serialize, Clone, Debug)]
pub struct EntryResponse {
    pub id:      u64,
    pub content: Value
}


impl Entry {
    pub fn from_row(row: &postgres::Row) -> EntryResponse {
        EntryResponse {
            id: row.get::<_, i64>("id") as u64,
            content: from_str::<Value>(&row.get::<_, String>("content")).unwrap(),
        }
    }

    pub fn get_one(c: &mut postgres::Client, id: u64, namespace: String) -> Result<EntryResponse, u64> {
        match c.query_one(
            "SELECT * FROM entries WHERE id = $1 AND namespace = $2",
            &[&(id as i64), &namespace]
        ) {
            Ok(row) => Ok(Self::from_row(&row)),
            Err(_) => Err(id),
        }
    }

    /*
    pub fn get_all(c: &mut postgres::Client, namespace: String) -> Vec<EntryResponse> {
        c.query(
            "SELECT * FROM entries WHERE namespace = $1",
            &[&namespace]
        )
        .unwrap()
        .iter()
        .map(|row| Self::from_row(row))
        .collect()
    }
    */

    pub fn get_page(c: &mut postgres::Client, namespace: String, page: u32, page_size: u16) -> Vec<EntryResponse> {
        c.query(
            "SELECT * FROM entries WHERE namespace = $1 \
             ORDER BY id ASC LIMIT $2 OFFSET $3",
            &[&namespace, &(page_size as i64), &(page as i64 * page_size as i64)]
        )
        .unwrap()
        .iter()
        .map(|row| Self::from_row(row))
        .collect()
    }

    pub fn insert(&self, c: &mut postgres::Client, namespace: String) -> u64 {
        c.query_one(
            "INSERT INTO entries (namespace, content) VALUES ($1, $2) RETURNING id",
            &[&namespace, &to_string(&self.0).unwrap()]
        )
        .expect("Failed to insert item!")
        .get::<_, i64>("id") as u64
    }

    pub fn insert_raw(c: &mut postgres::Client, namespace: String, entry: &Value) -> u64 {
        c.query_one(
            "INSERT INTO entries (namespace, content) VALUES ($1, $2) RETURNING id",
            &[&namespace, &to_string(entry).unwrap()]
        )
        .expect("Failed to insert item!")
        .get::<_, i64>("id") as u64
    }

    pub fn put(&self, c: &mut postgres::Client, id: u64, namespace: String) -> u64 {
        c.query_one(
            "INSERT INTO entries (id, namespace, content) VALUES ($1, $2, $3) ON CONFLICT (id) \
            DO UPDATE SET namespace = EXCLUDED.namespace, content = EXCLUDED.content RETURNING id",
            &[&(id as i64), &namespace, &to_string(&self.0).unwrap()]
        )
        .unwrap().get::<_, i64>("id") as u64
    }

    pub fn delete_all(c: &mut postgres::Client, namespace: String) -> u64 {
        c.query_one(
            "WITH rows as (DELETE FROM entries WHERE namespace = $1 RETURNING *) \
            SELECT COUNT(*) FROM rows",
            &[&namespace]
        )
        .expect("Fatal error on deletion!")
        .get::<_, i64>("count") as u64
    }

    pub fn delete_one(c: &mut postgres::Client, id: u64, namespace: String) -> Result<u64, u64> {
        match c.query_one(
            "DELETE FROM entries WHERE id = $1 AND namespace = $2 RETURNING id",
            &[&(id as i64), &namespace]
        ) {
            Ok(_) => Ok(id),
            Err(_) => Err(id)
        }
    }
}


#[rocket::async_trait]
impl<'r> FromData<'r> for Entry {
    type Error = ();

    async fn from_data(req: &'r Request<'_>, data: Data) -> Outcome<Self, ()> {
        // Ensure the content type is correct before opening the data.
        if req.content_type() != Some(&ContentType::JSON) {
            return Outcome::Forward(data);
        }

        // @Hack Here we forward to another handler if data is a list. We have to use
        // request routing info to avoid forward second time. We need this instead of
        // simpler handling that existed before, because all proper error handling below
        // is then unaccessible for second handler, which makes it much less usable.
        //
        // @RemoveLater Previous code:
        //if data.peek(1).await == b"[" {
        //    return Outcome::Forward(data);
        //}
        //
        match req.route().expect("Route is empty during handling request body!").rank {
            1 => return Outcome::Forward(data),
            _ => ()
        };
        // This is an optional header which defines the size in bytes of data sent
        // in the request. By default size is capped at 1MB, and if you want to send
        // bigger data, you must provide X-Content-Length. If body of the request is
        // any longer than provided length, server will return an error.
        let limit = match req.headers().get_one("X-Content-Length") {
            Some(raw_size) => match raw_size.parse::<u32>() {
                Ok(size) => size.bytes(),
                // If we got bad data we better off making it clear
                // than silently setting default buffer limit.
                Err(e) => {
                    // Store error message.
                    req.local_cache(|| ErrorMessage(Some(json!({
                        "code":    "err_content_length_parse",
                        "message": format!("Couldn't parse X-Content-Length with error: '{}'!", e)
                    }))));
                    return Outcome::Failure((Status::BadRequest, ()));
                }
            },
            None => DEFAULT_BUFFER_LIMIT.bytes()
        };

        match data.open(limit).into_string().await {
            Ok(string) => match string {
                s if s.is_complete() => match from_str::<Value>(&s) {
                    // Return successfully.
                    Ok(valid_data) => Outcome::Success(Entry(valid_data)),
                    Err(e) => {
                        req.local_cache(|| ErrorMessage(Some(json!({
                            "code":    "err_request_body_parse",
                            "message": format!("Couldn't parse request body into proper JSON with error: '{}'!", e)
                        }))));
                        Outcome::Failure((Status::BadRequest, ()))
                    }
                },
                // Here we handle error that indicates "too big buffer". We don't re-use an actual
                // error because it contains message in the format '<some message>: "<body>"' and
                // this is not good, because server needs to write the whole body (as in buffer) back,
                // which, in addition, makes error message unreadable.
                _ => {
                    req.local_cache(|| ErrorMessage(Some(json!({
                        "code":    "err_buffer_too_large",
                        "message": "Couldn't parse request body, it's too large! Default accepted size is 1MB. \
                            Consider using X-Content-Length header to set expected buffer size."
                    }))));
                    Outcome::Failure((Status::BadRequest, ()))
                }
            },
            Err(e) => {
                req.local_cache(|| ErrorMessage(Some(json!({
                    "code":    "err_request_body_read",
                    "message": format!("Couldn't read request body into string with error: '{}'!", e)
                }))));
                Outcome::Failure((Status::BadRequest, ()))
            }
        }
    }
}

