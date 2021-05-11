#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


#[cfg(test)] mod tests;


mod model;
mod health;
mod errors;
mod entries;
mod namespace;
mod pagination;
mod responders;


#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    #[cfg(not(debug_assertions))] println!("Voyeur is starting..");
    rocket::build()
        .mount("/api/v1/entries", routes![
            entries::get_entry_by_id,
            entries::get_query_content,
            entries::get_paginated_entries,
            entries::create_one_entry,
            entries::create_many_entries,
            entries::update_entry_by_id,
            entries::delete_all_entries,
            entries::delete_entry_by_id,
        ])
        .mount("/api/v1/health", routes![
            health::health_check_handler
        ])
        // API V1 error handlers
        .register("/api/v1", catchers![
            entries::handle_bad_request_errors,
        ])
        // Databases
        .attach(model::ApiDatabase::fairing())
}

