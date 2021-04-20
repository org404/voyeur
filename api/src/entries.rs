use rocket_contrib::json::{Json, JsonValue};
use crate::responders::CustomResponder;
use crate::model::{ApiDatabase, Entry};
use crate::pagination::PageSize;
use crate::namespace::Namespace;
use crate::errors::ErrorMessage;
use rocket::Request;


/// This endpoint is used to receive a single entry by ID (url path /<id> of type unsigned
/// 64-bit integer). Entry is an object containing id and content, example: {"id": 4, "content":
/// <your_json>}. For this endpoint you must provide namespace (url argument <namespace> or header
/// "X-Namespace", of type <String>).
#[get("/<id>")]
pub async fn get_entry_by_id(namespace: Namespace, id: u64, conn: ApiDatabase) -> CustomResponder {
    // @Robustness: This copy is required due to the way we handle errors right now.
    let namespace_copy = namespace.0.clone();

    match conn.run(move |c| Entry::get_one(c, id, namespace.0)).await {
        Ok(entry) => CustomResponder::Ok(json!({
            "code": "no_message",
            "namespace": &namespace_copy,
            "data": entry
        })),
        Err(id) => CustomResponder::BadRequest(json!({
            "code": "error_sql_get_one_by_id",
            "message": format!("Entry with ID '{}' does not exist!", id),
            "namespace": &namespace_copy,
            "id": id,
        }))
    }
}


/// This endpoint is used to receive a paginated JSON array of entries. Entry is an object
/// containing id and content, example: {"id": 4, "content": <your_json>}. For this endpoint
/// you must provide namespace (url argument <namespace> or header "X-Namespace", of type <String>)
/// and page (url argument <page>, of type unsigned 32-bit integer) values. Optionally, you can
/// specify a page size (url argument <page_size> or header "X-PAGE-SIZE", of type unsigned 16-bit
/// integer).
#[get("/?<page>")]
pub async fn get_paginated_entries(namespace: Namespace, page: u32, page_size: PageSize, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "no_message",
        "namespace": &namespace.0,
        "page_number": page.clone(),
        "page_size": page_size.0.clone(),
        "data": conn.run(
            move |c| Entry::get_page(c, namespace.0, page, page_size.0)
        ).await
    })
}


/// This endpoint is used to create an entry from your data. Body of the request must be a valid
/// JSON object*, so it can be recongnized by handler and interpreted for further dumping/loading.
/// For this endpoint you must provide namespace (url argument <namespace> or header "X-Namespace",
/// of type <String>). In addition to message code and message, correct response will contain ID of
/// the created entry.
///
/// * - Note, to allow storing multiple entries with single request, this handler ignores data that
///     looks like JSON array (see next handler).
#[post("/", format = "application/json", data = "<entry>", rank = 1)]
pub async fn create_one_entry(namespace: Namespace, entry: Entry, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_one_item_ok",
        "message": "Successfully created new entry!",
        "item_id": conn.run(move |c| entry.insert(c, namespace.0)).await
    })
}


/// This endpoint is used to create multiple entries from provided data. Body of the request must
/// be a valid JSON array containing any valid JSON objects. This array will be treated as a list
/// of entries to create. For this endpoint you must provide namespace (url argument <namespace>
/// or header "X-Namespace", of type <String>). In addition to message code and message, correct
/// response will contain a list of IDs of created entries.
#[post("/", format = "application/json", data = "<entries>", rank = 2)]
pub async fn create_many_entries(namespace: Namespace, entries: Json<Vec<Entry>>, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_many_items_ok",
        "message": "Successfully created multiple entries!",
        "item_ids": conn.run(
            move |c| entries.into_inner()
                .iter()
                .map(|entry| entry.insert(c, namespace.0.clone()))
                .collect::<Vec<u64>>()
        ).await
    })
}


/// This endpoint is used to update or create new entry with certain ID. Body of the request must
/// be a valid JSON objects, so it can be recongnized by handler and interpreted for further
/// dumping/loading. For this endpoint you must provide namespace (url argument <namespace>
/// or header "X-Namespace", of type <String>). In addition to message code and message, correct
/// response will contain ID of the put entry.
#[put("/<id>", format = "application/json", data = "<entry>")]
pub async fn update_entry_by_id(id: u64, namespace: Namespace, entry: Entry, conn: ApiDatabase) -> JsonValue {
    // TODO: This should return an error if the object exists but namespace is different, instead of updating (?).
    json!({
        "code": "info_item_put_ok",
        "message": "Successfully updated/created entry!",
        "item_id": conn.run(move |c| entry.put(c, id, namespace.0)).await
    })
}


/// This endpoint is used to delete all entries of certain namespace. For this endpoint you must
/// provide namespace (url argument <namespace> or header "X-Namespace", of type <String>). In
/// addition to message code and message, correct response will contain namespace itself and total
/// amount of deleted entries.
#[delete("/")]
pub async fn delete_all_entries(namespace: Namespace, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_delete_entries_ok",
        "message": format!("Successfully deleted all entries for namespace '{}'!", &namespace.0),
        "namespace": &namespace.0,
        "amount": conn.run(|c| Entry::delete_all(c, namespace.0)).await
    })
}


/// This endpoint is used to delete single entry by ID of certain namespace. For this endpoint you
/// must provide ID (url argument <id> of type unsigned 64-bit integer) namespace (url argument
/// <namespace> or header "X-Namespace", of type <String>). In addition to message code and message,
/// correct response will contain namespace itself and ID of the deleted entry.
#[delete("/<id>")]
pub async fn delete_entry_by_id(namespace: Namespace, id: u64, conn: ApiDatabase) -> CustomResponder {
    // @Robustness: This copy is required due to the way we handle errors right now.
    let namespace_copy = namespace.0.clone();

    match conn.run(move |c| Entry::delete_one(c, id, namespace.0)).await {
        Ok(id) => CustomResponder::Ok(json!({
            "code": "info_delete_entry_ok",
            "message": format!("Successfully deleted an entry of ID '{}' for namespace '{}'!", &id, &namespace_copy),
            "namespace": &namespace_copy,
            "id": id,
        })),
        Err(id) => CustomResponder::BadRequest(json!({
            "code": "error_sql_get_one_by_id",
            "message": format!("Entry with ID '{}' does not exist!", id),
            "namespace": &namespace_copy,
            "id": id,
        }))
    }
}


/// This endpoint is used to provide better error information and handle 400 http code errors with json
/// response, e.g. empty namespace value. It responds with 400 error code JSON message, containing short
/// code for an error to easily differentiate them without having to compare whole message (sometimes it's
/// not even possible to do comparsion without regex). Also, response contains error message itself and
/// some additional info if it's needed.
#[catch(400)]
pub fn handle_bad_request_errors(req: &Request) -> CustomResponder {
    match req.local_cache(|| ErrorMessage(None)) {
        ErrorMessage(Some(v)) => CustomResponder::BadRequest(v.clone()),
        // Default response
        ErrorMessage(None) => CustomResponder::UnknownError(json!({
            "code":    "err_unknown_error",
            "message": "Some unknown (unhandled) error occured! Please, report the bug by filing an issue.",
        }))
    }
}

