use crate::errors::ErrorMessage;
use rocket::{request, Request};
use rocket::http::Status;


/// Value which allows to access namespace value.
pub struct Namespace(pub String);


// Allows a route to access good namespace value.
#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for Namespace {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Extract value from header or url.
        let namespace = match req.headers().get_one("X-Namespace") {
            Some(value) => value.to_string(),
            // Headers are empty, so we look for url argument.
            None => match req.query_value::<&str>("namespace") {
                // This returns some result (of parsed value).
                Some(unparsed_value) => match unparsed_value {
                    // Return token if parsed correctly.
                    Ok(value) => value.to_string(),
                    // Return empty string if failed to parse.
                    Err(_) => "".to_string()
                },
                // Return empty string immediately if there is no argument.
                None => "".to_string(),
            }
        };

        match namespace {
            v if v.is_empty() => {
                // Store error message.
                req.local_cache(|| ErrorMessage(Some(json!({
                    "code":    "err_namespace_empty",
                    "message": "You must provide 'X-Namespace' header or 'namespace' URL argument with request!",
                }))));
                // Forward to error catcher.
                request::Outcome::Failure((Status::BadRequest, ()))
            },
            v if v.len() > 64 => {
                // Store error message.
                req.local_cache(|| ErrorMessage(Some(json!({
                    "code":      "err_namespace_long",
                    "message":   format!("Provided namespace value is too big (max is 64 characters, received {})!", v.len()),
                    "namespace": v,
                }))));
                // Forward to error catcher.
                request::Outcome::Failure((Status::BadRequest, ()))
            },
            // Good namespace is ready for use.
            v => request::Outcome::Success(Namespace(v))
        }
    }
}

