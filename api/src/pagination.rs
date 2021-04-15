use rocket::request::{Outcome, Request, FromRequest};
use rocket::http::Status;
use serde::Serialize;


static DEFAULT_PAGE_SIZE: u16 = 25;


#[derive(Debug, Clone, Serialize)]
pub struct PageSize(pub u16);


#[derive(Debug)]
pub enum HeaderError {
    PageSizeBad,
    PageSizeZero,
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for PageSize {
    type Error = HeaderError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // First we check for header value.
        match req.headers().get_one("X-Page-Size") {
            Some(v) => match v.parse::<u16>() {
                Ok(page_size) => match page_size {
                    // Forbid 0 page size.
                    n if n > 0 => Outcome::Success(PageSize(n)),
                    // Here we return an error without checking for url argument,
                    // because header was provided, so better to provide error.
                    _ => Outcome::Failure((Status::PreconditionFailed, HeaderError::PageSizeZero))
                },
                // Here we return an error without checking for url argument,
                // because header was provided, so better to provide error.
                Err(_) => Outcome::Failure((Status::PreconditionFailed, HeaderError::PageSizeBad))
            },
            // Fallback to URL arguments.
            None => match req.query_value::<u16>("page_size") {
                // This returns some result (of parsed value).
                Some(unparsed_value) => match unparsed_value {
                    Ok(page_size) => match page_size {
                        // Forbid 0 page size.
                        n if n > 0 => Outcome::Success(PageSize(n)),
                        _ => Outcome::Failure((Status::PreconditionFailed, HeaderError::PageSizeZero))
                    },
                    Err(_) => Outcome::Failure((Status::PreconditionFailed, HeaderError::PageSizeBad))
                },
                // Return default size if there is no argument.
                None => Outcome::Success(PageSize(DEFAULT_PAGE_SIZE))
            }
        }
    }
}

