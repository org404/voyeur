use rocket_contrib::json::JsonValue;


#[derive(Responder)]
pub enum CustomResponder {
    #[response(status = 200, content_type = "json")]
    Ok(JsonValue),
    #[response(status = 400, content_type = "json")]
    BadRequest(JsonValue),
    #[response(status = 500, content_type = "json")]
    UnknownError(JsonValue),
}

