use rocket::request::{Outcome, Request, FromRequest};
use crate::errors::ErrorMessage;
use rocket::http::Status;
use serde::Serialize;


static DEFAULT_PAGE_SIZE: u16 = 25;


#[derive(Debug, Clone, Serialize)]
pub struct PageSize(pub u16);


#[rocket::async_trait]
impl<'r> FromRequest<'r> for PageSize {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, ()> {
        // First we check for header value.
        match req.headers().get_one("X-Page-Size") {
            Some(v) => match v.parse::<u16>() {
                Ok(page_size) => match page_size {
                    // Forbid 0 page size.
                    n if n > 0 => Outcome::Success(PageSize(n)),
                    // Here we return an error without checking for url argument,
                    // because header was provided, so better to provide error.
                    // Returning error for 0 page size.
                    _ => {
                        // Store error message.
                        req.local_cache(|| ErrorMessage(Some(json!({
                            "code":    "err_page_size_zero",
                            "message": "You must provide non-zero value for page size request!"
                        }))));
                        Outcome::Failure((Status::BadRequest, ()))
                    }
                },
                // Here we return an error without checking for url argument,
                // because header was provided, so better to provide error.
                Err(e) => {
                    // Store error message.
                    req.local_cache(|| ErrorMessage(Some(json!({
                        "code":    "err_page_size_parsing",
                        "message": format!("Couldn't parse page size from header with error: '{}'!", e)
                    }))));
                    Outcome::Failure((Status::BadRequest, ()))
                }
            },
            // Fallback to URL arguments.
            None => match req.query_value::<u16>("page_size") {
                // This returns some result (of parsed value).
                Some(unparsed_value) => match unparsed_value {
                    Ok(page_size) => match page_size {
                        // Forbid 0 page size.
                        n if n > 0 => Outcome::Success(PageSize(n)),
                        _ => {
                            // Store error message.
                            req.local_cache(|| ErrorMessage(Some(json!({
                                "code":    "err_page_size_zero",
                                "message": "You must provide non-zero value for page size request!"
                            }))));
                            Outcome::Failure((Status::BadRequest, ()))
                        }
                    },
                    Err(e) => {
                        // Store error message.
                        req.local_cache(|| ErrorMessage(Some(json!({
                            "code":    "err_page_size_parsing",
                            "message": format!("Couldn't parse page size from url argument with error: '{}'!", e)
                        }))));
                        Outcome::Failure((Status::BadRequest, ()))
                    }
                },
                // Return default size if there is no argument.
                None => Outcome::Success(PageSize(DEFAULT_PAGE_SIZE))
            }
        }
    }
}

