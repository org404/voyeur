use rocket::local::asynchronous::Client;
use rocket::http::Status;


#[rocket::async_test]
async fn test_health_endpoint() {
    let client = Client::tracked(super::rocket()).await.unwrap();

    let resp = client.get("/api/v1/health").dispatch().await;
    assert_eq!(resp.status(), Status::Ok);

    let rbody = resp.into_string().await.unwrap();
    let value = json!({"code": "info_status_check_ok"}).to_string();
    assert_eq!(rbody, value);
}

