use rocket::http::{ContentType, Status, Header};
use rocket::local::asynchronous::Client;
use rocket::tokio;
use super::rocket;


#[rocket::async_test]
async fn test_bad() {
    let client = Client::tracked(rocket()).await.unwrap();

    {
        // Here we test entry ID bad values. Note that we don't need to add
        // request body here, because url argument is matched before.
        let (r1, r2, r3, r4, r5) = tokio::join!(
            client.put("/api/v1/entries/-1")
                .header(Header::new("X-Namespace", "a"))
                .header(ContentType::JSON).dispatch(),
            client.put("/api/v1/entries/0.12345")
                .header(Header::new("X-Namespace", "a"))
                .header(ContentType::JSON).dispatch(),
            client.put("/api/v1/entries/a")
                .header(Header::new("X-Namespace", "a"))
                .header(ContentType::JSON).dispatch(),
            client.put(format!("/api/v1/entries/{}", u64::MAX as u128 + 1))
                .header(Header::new("X-Namespace", "a"))
                .header(ContentType::JSON).dispatch(),
            client.put("/api/v1/entries/0x1122334455")
                .header(Header::new("X-Namespace", "a"))
                .header(ContentType::JSON).dispatch()
        );

        assert_eq!(r1.content_type(), Some(ContentType::HTML));
        assert_eq!(r2.content_type(), Some(ContentType::HTML));
        assert_eq!(r3.content_type(), Some(ContentType::HTML));
        assert_eq!(r4.content_type(), Some(ContentType::HTML));
        assert_eq!(r5.content_type(), Some(ContentType::HTML));

        assert_eq!(r1.status(), Status::NotFound);
        assert_eq!(r2.status(), Status::NotFound);
        assert_eq!(r3.status(), Status::NotFound);
        assert_eq!(r4.status(), Status::NotFound);
        assert_eq!(r5.status(), Status::NotFound);
    }
}

