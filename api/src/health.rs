use rocket_contrib::json::JsonValue;


#[get("/")]
pub async fn health_check_handler() -> JsonValue {
    // TODO: more sophisticated healthcheck
    json!({
        "code": "info_status_check_ok"
    })
}

