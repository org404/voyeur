use rocket::{request, Request};


/// Value which allows to access namespace value.
pub struct Namespace(pub String);

/// Struct to hold red path value of namespace.
pub struct BadNamespace(pub Option<String>);


// Allows a route to access good namespace value.
#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for Namespace {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ()> {
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
            v if v.is_empty() | (v.len() > 64) => {
                // Store bad namespace value
                req.local_cache(|| BadNamespace(Some(v)));
                // Forward to error handler which is below in the rank ladder.
                request::Outcome::Forward(())
            },
            v => {
                // Good namespace is ready for use.
                request::Outcome::Success(Namespace(v))
            }
        }
    }
}


// Allows a route to access bad namespace value.
#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for BadNamespace {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ()> {
        match req.local_cache(|| BadNamespace(None)) {
            BadNamespace(Some(value)) => request::Outcome::Success(BadNamespace(Some(value.to_string()))),
            BadNamespace(None) => request::Outcome::Success(BadNamespace(None)),
        }
    }
}

