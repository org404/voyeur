use rocket::http::{ContentType, Status, Header};
use rocket::local::asynchronous::Client;
use rocket::tokio;
use super::rocket;


#[rocket::async_test]
async fn test_bad() {
    let client = Client::tracked(rocket()).await.unwrap();

    {
        // Test bad page value.
        let (r1, r2, r3, r4, r5) = tokio::join!(
            client.get("/api/v1/entries?namespace=a&page=").dispatch(),
            client.get("/api/v1/entries?namespace=a&page=a").dispatch(),
            client.get("/api/v1/entries?page=0x1122334455")
                .header(Header::new("X-Namespace", "a")).dispatch(),
            client.get("/api/v1/entries?page=-1")
                .header(Header::new("X-Namespace", "a")).dispatch(),
            client.get(format!("/api/v1/entries?page={}", u64::MAX as u128 + 1))
                .header(Header::new("X-Namespace", "a")).dispatch()
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

    {
        // Test bad page size value.
        let (r1, r2, r3, r4, r5, r6, r7) = tokio::join!(
            client.get("/api/v1/entries?namespace=a&page=0&page_size=").dispatch(),
            client.get("/api/v1/entries?namespace=a&page=0&page_size=-1").dispatch(),
            client.get(format!("/api/v1/entries?namespace=a&page=0&page_size={}", u16::MAX as u32 + 1)).dispatch(),
            client.get(format!("/api/v1/entries?namespace=a&page=0&page_size={}", u64::MAX)).dispatch(),
            client.get("/api/v1/entries?page=0")
                .header(Header::new("X-Page-Size", ""))
                .header(Header::new("X-Namespace", "a")).dispatch(),
            client.get("/api/v1/entries?page=0")
                .header(Header::new("X-Page-Size", "a"))
                .header(Header::new("X-Namespace", "a")).dispatch(),
            client.get("/api/v1/entries?page=0")
                .header(Header::new("X-Page-Size", "0x1122334455"))
                .header(Header::new("X-Namespace", "a")).dispatch()
        );

        assert_eq!(r1.content_type(), Some(ContentType::JSON));
        assert_eq!(r2.content_type(), Some(ContentType::JSON));
        assert_eq!(r3.content_type(), Some(ContentType::JSON));
        assert_eq!(r4.content_type(), Some(ContentType::JSON));
        assert_eq!(r5.content_type(), Some(ContentType::JSON));
        assert_eq!(r6.content_type(), Some(ContentType::JSON));
        assert_eq!(r7.content_type(), Some(ContentType::JSON));

        assert_eq!(r1.status(), Status::BadRequest);
        assert_eq!(r2.status(), Status::BadRequest);
        assert_eq!(r3.status(), Status::BadRequest);
        assert_eq!(r4.status(), Status::BadRequest);
        assert_eq!(r5.status(), Status::BadRequest);
        assert_eq!(r6.status(), Status::BadRequest);
        assert_eq!(r7.status(), Status::BadRequest);

        let (s1, s2, s3, s4, s5, s6, s7) = rocket::tokio::join!(
            r1.into_string(), r2.into_string(), r3.into_string(),
            r4.into_string(), r5.into_string(), r6.into_string(),
            r7.into_string()
        );

        assert_eq!(s1, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from url argument with error: '1 errors:\ninvalid integer: cannot parse integer from empty string'!"
        }).to_string()));

        assert_eq!(s2, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from url argument with error: '1 errors:\ninvalid integer: invalid digit found in string'!"
        }).to_string()));

        assert_eq!(s3, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from url argument with error: '1 errors:\ninvalid integer: number too large to fit in target type'!"
        }).to_string()));

        assert_eq!(s4, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from url argument with error: '1 errors:\ninvalid integer: number too large to fit in target type'!"
        }).to_string()));

        assert_eq!(s5, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from header with error: 'cannot parse integer from empty string'!"
        }).to_string()));

        assert_eq!(s6, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from header with error: 'invalid digit found in string'!"
        }).to_string()));

        assert_eq!(s7, Some(json!({
            "code": "err_page_size_parsing",
            "message": "Couldn't parse page size from header with error: 'invalid digit found in string'!"
        }).to_string()));
    }
}

