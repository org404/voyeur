use rocket::local::asynchronous::Client;
use rocket::http::{ContentType, Status};
use crate::model::{ApiDatabase, Entry};
use serde_json::{from_str, Value};
use super::rocket;

// @FromRocketExamples:
// We use a lock to synchronize between tests so DB operations don't collide.
// For now. In the future, we'll have a nice way to run each test in a DB
// transaction so we can regain concurrency.
static DB_LOCK: parking_lot::Mutex<()> = parking_lot::const_mutex(());


// Macro for running async block in a blocking way.
macro_rules! run_test {
    (|$client:ident, $conn:ident| $block:expr) => ({
        let _lock = DB_LOCK.lock();

        rocket::async_test(async move {
            let $client = Client::tracked(rocket()).await.expect("Rocket client");
            let db = ApiDatabase::get_one($client.rocket()).await;
            let $conn = db.expect("failed to get database connection for testing");
            // Note: this deletes all entries on 'test_name_alpha' namespace to make
            //       tests more consistent and easier to write.
            $conn.run(|c| Entry::delete_all(c, "test_name_alpha".to_string())).await;

            $block
        })
    })
}


#[test]
fn test_suit_1() {
    run_test!(|client, _conn| {
        // Store ids for comparison.
        let id1: u64;
        let id2: u64;

        {
            // Adding entry #1 ...
            let r = client.post("/api/v1/entries?namespace=test_name_alpha").header(ContentType::JSON)
                .body("{\"text\": \"This is some test data!\"}").dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "info_one_item_ok");

            // Save id for later.
            id1 = body.get("item_id")
                .expect("Expected response to contain 'item_id' field..")
                .as_u64().expect("Failed to parse 'item_id' value as u64..");
        }

        {
            // Adding entry #2 ...
            let r = client.post("/api/v1/entries?namespace=test_name_alpha").header(ContentType::JSON)
                .body("{\"text\": \"This is second test data!\"}").dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "info_one_item_ok");

            // Save id for later.
            id2 = body.get("item_id")
                .expect("Expected response to contain 'item_id' field..")
                .as_u64().expect("Failed to parse 'item_id' value as u64..");
        }

        {
            // Query page of entries ...
            let r = client.get("/api/v1/entries?page=0&namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");
            let data = body.get("data")
                .expect("Expected response to contain 'data' field..")
                .as_array().expect("Expected 'data' field to be a JSON array..");

            assert_eq!(code, "no_message");
            assert_eq!(data.len(), 2);

            let _id1 = data[0].get("id")
                .expect("Expected response to contain 'id' field..")
                .as_u64().expect("Failed to parse 'id' value as u64..");
            let text1 = data[0]
                .get("content").expect("Expected response to contain 'content' field..")
                .get("text").expect("Expected 'content' to contain 'text' field..")
                .as_str().expect("Expected 'text' field to be a string..");

            assert_eq!(_id1, id1);
            assert_eq!(text1, "This is some test data!");

            let _id2 = data[1].get("id")
                .expect("Expected response to contain 'id' field..")
                .as_u64().expect("Failed to parse 'id' value as u64..");
            let text2 = data[1]
                .get("content").expect("Expected response to contain 'content' field..")
                .get("text").expect("Expected 'content' to contain 'text' field..")
                .as_str().expect("Expected 'text' field to be a string..");

            assert_eq!(_id2, id2);
            assert_eq!(text2, "This is second test data!");
        }

        {
            // Delete all entries ...
            let r = client.delete("/api/v1/entries?namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "info_delete_entries_ok");

            let amount = body.get("amount")
                .expect("Expected response to contain 'amount' field..")
                .as_u64().expect("Failed to parse 'amount' field as u64..");

            assert_eq!(amount, 2);
        }

        {
            // Query page of entries again to verify it's empty ...
            let r = client.get("/api/v1/entries?page=0&namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");
            let data = body.get("data")
                .expect("Expected response to contain 'data' field..")
                .as_array().expect("Expected 'data' field to be a JSON array..");

            assert_eq!(code, "no_message");
            assert_eq!(data.len(), 0);
        }
    })
}


#[test]
fn test_suit_2() {
    run_test!(|client, _conn| {
        let test_id: u64;

        {
            // Creating 3 entries ...
            let r = client.post("/api/v1/entries?namespace=test_name_alpha").header(ContentType::JSON)
                .body("[{\"log\": \"Value 1\"}, {\"log\": \"Value 2\"}, {\"log\": \"Last value\"}]").dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "info_many_items_ok");

            // Save id for later.
            let ids = body.get("item_ids")
                .expect("Expected response to contain 'item_ids' field..")
                .as_array().expect("Failed to parse 'item_ids' as an array..");

            assert_eq!(ids.len(), 3);

            test_id = ids[0].as_u64().expect("Failed to parse 'item_ids[0]' as u64..");
        }

        {
            // Put entry in the place of existing one ...
            let r = client.put(format!("/api/v1/entries/{}?namespace=test_name_alpha", test_id))
                .header(ContentType::JSON).body("{\"log\": \"New Value\"}").dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "info_put_item_ok");
        }

        {
            // Query entry by id ...
            let r = client.get(format!("/api/v1/entries/{}?namespace=test_name_alpha", test_id))
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "no_message");

            let content = body.get("data")
                .expect("Expected response to contain 'data' field..")
                .get("content").expect("Expected 'data' to contain 'content' field..");
            let log = content.get("log")
                .expect("Expected 'content' to contain 'log' field..")
                .as_str().expect("Expected 'log' to be a string..");

            assert_eq!(log, "New Value");
        }

        {
            // Delete entry by id ...
            let r = client.delete(format!("/api/v1/entries/{}?namespace=test_name_alpha", test_id))
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string..");

            assert_eq!(code, "info_delete_entry_ok");
        }

        {
            // Query page of entries to verify that previous request worked ...
            let r = client.get("/api/v1/entries?page=0&namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap())
                .expect("Failed to read request body as JSON..");
            let code = body.get("code")
                .expect("Expected response to contain 'code' field..")
                .as_str().expect("Expected 'code' field to be a string");
            let data = body.get("data")
                .expect("Expected response to contain 'data' field..")
                .as_array().expect("Failed to parse 'data' as an array..");

            assert_eq!(code, "no_message");
            assert_eq!(data.len(), 2);

            let _ = data.iter().map(|item| {
                let id = item.get("content")
                    .expect("Expected response to contain 'content' field..")
                    .get("id").expect("Expected 'content' to contain 'id' field..")
                    .as_u64().expect("Failed to parse 'id' as u64..");
                assert_ne!(test_id.clone(), id);
            });
        }

        // Note:
        //     Macro deletes all existing entries on the start of the block, which
        //     means we don't have to clear anything here. Unless, of course, we want
        //     to test deletion here too.
        //                                                         - andrew, April 26 2021
    })
}

