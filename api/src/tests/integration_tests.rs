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

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "info_one_item_ok");

            // Save id for later.
            id1 = body.get("item_id").unwrap().as_u64().unwrap();
        }

        {
            // Adding entry #2 ...
            let r = client.post("/api/v1/entries?namespace=test_name_alpha").header(ContentType::JSON)
                .body("{\"text\": \"This is second test data!\"}").dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "info_one_item_ok");

            // Save id for later.
            id2 = body.get("item_id").unwrap().as_u64().unwrap();
        }

        {
            // Query page of entries ...
            let r = client.get("/api/v1/entries?page=0&namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();
            let data = body.get("data").unwrap().as_array().unwrap();

            assert_eq!(code, "no_message");
            assert_eq!(data.len(), 2);

            let _id1 = data[0].get("id").unwrap().as_u64();
            let text1 = data[0].get("content").unwrap().get("text").unwrap().as_str();

            assert_eq!(_id1, Some(id1));
            assert_eq!(text1, Some("This is some test data!"));

            let _id2 = data[1].get("id").unwrap().as_u64();
            let text2 = data[1].get("content").unwrap().get("text").unwrap().as_str();

            assert_eq!(_id2, Some(id2));
            assert_eq!(text2, Some("This is second test data!"));
        }

        {
            // Delete all entries ...
            let r = client.delete("/api/v1/entries?namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "info_delete_entries_ok");

            let amount = body.get("amount").unwrap().as_u64().unwrap();

            assert_eq!(amount, 2);
        }

        {
            // Query page of entries again to verify it's empty ...
            let r = client.get("/api/v1/entries?page=0&namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();
            let data = body.get("data").unwrap().as_array().unwrap();

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

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "info_many_items_ok");

            // Save id for later.
            let ids = body.get("item_ids").unwrap().as_array().unwrap();

            assert_eq!(ids.len(), 3);

            test_id = ids[0].as_u64().unwrap();
        }

        {
            // Put entry in the place of existing one ...
            let r = client.put(format!("/api/v1/entries/{}?namespace=test_name_alpha", test_id))
                .header(ContentType::JSON).body("{\"log\": \"New Value\"}").dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "info_put_item_ok");
        }

        {
            // Query entry by id ...
            let r = client.get(format!("/api/v1/entries/{}?namespace=test_name_alpha", test_id))
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "no_message");

            let content = body.get("data").unwrap().get("content").unwrap();
            let log = content.get("log").unwrap().as_str().unwrap();

            assert_eq!(log, "New Value");
        }

        {
            // Delete entry by id ...
            let r = client.delete(format!("/api/v1/entries/{}?namespace=test_name_alpha", test_id))
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();

            assert_eq!(code, "info_delete_entry_ok");
        }

        {
            // Query page of entries to verify that previous request worked ...
            let r = client.get("/api/v1/entries?page=0&namespace=test_name_alpha")
                .header(ContentType::JSON).dispatch().await;

            assert_eq!(r.content_type(), Some(ContentType::JSON));
            assert_eq!(r.status(), Status::Ok);

            let body = from_str::<Value>(&r.into_string().await.unwrap()).unwrap();
            let code = body.get("code").unwrap().as_str().unwrap();
            let data = body.get("data").unwrap().as_array().unwrap();

            assert_eq!(code, "no_message");
            assert_eq!(data.len(), 2);

            let _ = data.iter().map(|item| {
                let raw_id = item.get("content").unwrap().get("id").unwrap();
                let id = raw_id.as_u64().unwrap();
                assert_ne!(test_id.clone(), id);
            });
        }
        // Note:
        // Macro deletes all existing entries on the start of the block, which
        // means we don't have to clear anything here. Unless, of course, we want
        // to test deletion.
    })
}

