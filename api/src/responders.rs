use rocket_contrib::json::JsonValue;


#[derive(Responder)]
pub enum Error {
    #[response(status = 400, content_type = "json")]
    BadRequest(JsonValue)
}

