use rocket::local::asynchronous::Client;
use rocket::http::{ContentType, Status};
use rocket::tokio;
use super::rocket;


#[rocket::async_test]
async fn test_bad() {
    let client = Client::tracked(rocket()).await.unwrap();

    {
        // Test bad body / headers.
        let (r1, r2, r3, r4, r5, r6, r7, r8) = tokio::join!(
            client.post("/api/v1/entries?namespace=a").dispatch(),
            client.post("/api/v1/entries?namespace=a").body("a=1").dispatch(),
            client.post("/api/v1/entries?namespace=a")
                .header(ContentType::HTML).dispatch(),
            client.post("/api/v1/entries?namespace=a")
                .header(ContentType::JSON).dispatch(),
            client.post("/api/v1/entries?namespace=a")
                .header(ContentType::JSON).body("[1 '']").dispatch(),
            client.post("/api/v1/entries?namespace=a")
                .header(ContentType::JSON).body("[1, 2, 3, 4, 5, #]").dispatch(),
            client.post("/api/v1/entries?namespace=a")
                .header(ContentType::JSON).body("[{\"a\": [(), ()]}]").dispatch(),
            // Create 2 MB list which won't fit in default buffer.
            client.post("/api/v1/entries?namespace=a")
                .header(ContentType::JSON).body(format!("[{}]", "1,".repeat(1024 * 1024))).dispatch(),
        );

        assert_eq!(r1.content_type(), Some(ContentType::HTML));
        assert_eq!(r2.content_type(), Some(ContentType::HTML));
        assert_eq!(r3.content_type(), Some(ContentType::HTML));
        // Parsing errors handled by us.
        assert_eq!(r4.content_type(), Some(ContentType::JSON));
        assert_eq!(r5.content_type(), Some(ContentType::JSON));
        assert_eq!(r6.content_type(), Some(ContentType::JSON));
        assert_eq!(r7.content_type(), Some(ContentType::JSON));
        assert_eq!(r8.content_type(), Some(ContentType::JSON));

        assert_eq!(r1.status(), Status::NotFound);
        assert_eq!(r2.status(), Status::NotFound);
        assert_eq!(r3.status(), Status::NotFound);
        assert_eq!(r4.status(), Status::BadRequest);
        assert_eq!(r5.status(), Status::BadRequest);
        assert_eq!(r6.status(), Status::BadRequest);
        assert_eq!(r7.status(), Status::BadRequest);
        assert_eq!(r8.status(), Status::BadRequest);

        let (s5, s6, s7, s8) = rocket::tokio::join!(
            r5.into_string(), r6.into_string(),
            r7.into_string(), r8.into_string()
        );

        assert_eq!(s5, Some(json!({
            "code": "err_request_body_parse",
            "message": "Couldn't parse request body into proper JSON with error: 'expected `,` or `]` at line 1 column 4'!"
        }).to_string()));

        assert_eq!(s6, Some(json!({
            "code": "err_request_body_parse",
            "message": "Couldn't parse request body into proper JSON with error: 'expected value at line 1 column 17'!"
        }).to_string()));

        assert_eq!(s7, Some(json!({
            "code": "err_request_body_parse",
            "message": "Couldn't parse request body into proper JSON with error: 'expected value at line 1 column 9'!"
        }).to_string()));

        assert_eq!(s8, Some(json!({
            "code": "err_buffer_too_large",
            "message": "Couldn't parse request body, it's too large! Default accepted size is 1MB. Consider using X-Content-Length header to set expected buffer size."
        }).to_string()));
    }
}

