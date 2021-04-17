use crate::namespace::{Namespace, BadNamespace};
use rocket_contrib::json::{Json, JsonValue};
use crate::model::{ApiDatabase, Entry};
use crate::pagination::PageSize;
use crate::responders::Error;


#[get("/?<page>", rank = 1)]
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


#[get("/", rank = 2)]
pub async fn get_all_entries(namespace: Namespace, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "no_message",
        "namespace": &namespace.0,
        "data": conn.run(|c| Entry::get_all(c, namespace.0)).await
    })
}


// Error handler for bad namespace value
#[get("/", rank = 3)]
pub fn handle_namespace_errors(namespace: BadNamespace) -> Error {
    match namespace.0 {
        Some(v) if v.is_empty() => Error::BadRequest(json!({
            "code":    "err_namespace_empty",
            "message": "You must provide 'X-Namespace' header or 'namespace' URL argument with request!",
        })),
        Some(v) => Error::BadRequest(json!({
            "code":      "err_namespace_long",
            "message":   format!("Provided namespace value is too big (max is 64 characters, received {})!", v.len()),
            "namespace": v,
        })),
        None => Error::BadRequest(json!({
            "code":    "err_namespace_empty",
            "message": "You must provide 'X-Namespace' header or 'namespace' URL argument with request!",
        }))
    }
}


#[post("/", format = "application/json", data = "<entry>", rank = 1)]
pub async fn create_one_entry(namespace: Namespace, entry: Entry, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_one_item_ok",
        "message": "Successfully created new entry!",
        "item_id": conn.run(move |c| entry.insert(c, namespace.0)).await
    })
}


#[post("/", format = "application/json", data = "<entries>", rank = 2)]
pub async fn create_many_entries(namespace: Namespace, entries: Json<Vec<Entry>>, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_many_items_ok",
        "message": "Successfully created multiple entries!",
        "item_ids": conn.run(
            move |c| entries.into_inner()
                .iter()
                .map(|entry| entry.insert(c, namespace.0.clone()))
                .collect::<Vec<u32>>()
        ).await
    })
}


#[put("/<id>", format = "application/json", data = "<entry>")]
pub async fn update_entry_by_id(id: u32, namespace: Namespace, entry: Entry, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_item_put_ok",
        "message": "Successfully updated/created entry!",
        "item_id": conn.run(move |c| entry.put(c, id, namespace.0)).await
    })
}


#[delete("/")]
pub async fn delete_all_entries(namespace: Namespace, conn: ApiDatabase) -> JsonValue {
    json!({
        "code": "info_delete_entries_ok",
        "message": format!("Successfully deleted all entries for namespace '{}'!", &namespace.0),
        "namespace": &namespace.0,
        "amount": conn.run(|c| Entry::delete_all(c, namespace.0)).await
    })
}

