use rocket::http::{ContentType, Status, Header};
use rocket::local::asynchronous::Client;
use rocket::tokio;
use super::rocket;


#[rocket::async_test]
async fn test_bad() {
    let client = Client::tracked(rocket()).await.unwrap();

    {
        // Test empty namespace value.
        let (r1, r2, r3) = tokio::join!(
            client.get("/api/v1/entries/1").dispatch(),
            client.get("/api/v1/entries?namespace=").dispatch(),
            client.get("/api/v1/entries").header(Header::new("X-Namespace", "")).dispatch()
        );

        assert_eq!(r1.content_type(), Some(ContentType::JSON));
        assert_eq!(r2.content_type(), Some(ContentType::JSON));
        assert_eq!(r3.content_type(), Some(ContentType::JSON));

        assert_eq!(r1.status(), Status::BadRequest);
        assert_eq!(r2.status(), Status::BadRequest);
        assert_eq!(r3.status(), Status::BadRequest);

        let (s1, s2, s3) = rocket::tokio::join!(
            r1.into_string(),
            r2.into_string(),
            r3.into_string()
        );
        let value = json!({
            "code": "err_namespace_empty",
            "message": "You must provide 'X-Namespace' header or 'namespace' URL argument with request!"
        }).to_string();

        assert_eq!(s1, Some(value.clone()));
        assert_eq!(s2, Some(value.clone()));
        assert_eq!(s3, Some(value.clone()));
    }

    {
        // Namespace value is too long.
        let (r1, r2) = tokio::join!(
            client.get("/api/v1/entries")
                .header(Header::new("X-Namespace", "This sample sentence was crafted to be exactly 65 characters long"))
                .dispatch(),
            client.get("/api/v1/entries?namespace=This%20sample%20sentence%20was%20crafted%20to%20be%20exactly%2065%20characters%20long")
                .dispatch()
        );

        assert_eq!(r1.content_type(), Some(ContentType::JSON));
        assert_eq!(r2.content_type(), Some(ContentType::JSON));

        assert_eq!(r1.status(), Status::BadRequest);
        assert_eq!(r2.status(), Status::BadRequest);

        let (s1, s2) = rocket::tokio::join!(
            r1.into_string(),
            r2.into_string()
        );
        let value = json!({
            "code": "err_namespace_long",
            "message": "Provided namespace value is too big (max is 64 characters, received 65)!",
            "namespace": "This sample sentence was crafted to be exactly 65 characters long"
        }).to_string();

        assert_eq!(s1, Some(value.clone()));
        assert_eq!(s2, Some(value.clone()));
    }

    {
        // Bad values for resource ID.
        let (r1, /*r2,*/ r3, r4, r5, r6) = tokio::join!(
            client.get("/api/v1/entries/-1?namespace=a").dispatch(),
            // TODO: @Question This might be an issue? How does postgres handle 0 id?
            //client.get("/api/v1/entries/0?namespace=a").dispatch(),
            client.get(format!("/api/v1/entries/{}?namespace=a", u64::MAX as u128 + 1)).dispatch(),
            client.get("/api/v1/entries/a").header(Header::new("X-Namespace", "a")).dispatch(),
            client.get("/api/v1/entries/0x194125").header(Header::new("X-Namespace", "a")).dispatch(),
            client.get("/api/v1/entries/0.12345").header(Header::new("X-Namespace", "a")).dispatch(),
        );

        assert_eq!(r1.content_type(), Some(ContentType::HTML));
        //assert_eq!(r2.content_type(), Some(ContentType::HTML));
        assert_eq!(r3.content_type(), Some(ContentType::HTML));
        assert_eq!(r4.content_type(), Some(ContentType::HTML));
        assert_eq!(r5.content_type(), Some(ContentType::HTML));
        assert_eq!(r6.content_type(), Some(ContentType::HTML));

        assert_eq!(r1.status(), Status::NotFound);
        //assert_eq!(r2.status(), Status::NotFound);
        assert_eq!(r3.status(), Status::NotFound);
        assert_eq!(r4.status(), Status::NotFound);
        assert_eq!(r5.status(), Status::NotFound);
        assert_eq!(r6.status(), Status::NotFound);
    }
}

