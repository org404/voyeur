#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


mod sql;
mod model;
mod health;
mod entries;
mod namespace;
mod pagination;
mod responders;


#[launch]
fn rocket() -> rocket::Rocket {
    println!("Voyeur is starting..");
    rocket::ignite()
        .mount("/api/v1/entries", routes![
            entries::handle_namespace_errors,
            entries::get_all_entries,
            entries::get_paginated_entries,
            entries::create_one_entry,
            entries::create_many_entries,
            entries::update_entry_by_id,
            entries::delete_all_entries
        ])
        .mount("/api/v1/health", routes![
            health::health_check_handler
        ])
        // Databases
        .attach(model::ApiDatabase::fairing())
}

