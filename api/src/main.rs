#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


mod sql;
mod model;
mod health;
mod namespace;
mod interface;
mod pagination;


#[launch]
fn rocket() -> rocket::Rocket {
    println!("Voyeur is starting..");
    rocket::ignite()
        .mount("/api/v1/logs", routes![
            interface::handle_namespace_errors,
            interface::get_all_entries,
            interface::get_paginated_entries,
            interface::create_one_entry,
            interface::create_many_entries,
            interface::update_entry_by_id,
            interface::delete_all_entries
        ])
        .mount("/api/v1/health", routes![
            health::health_check_handler
        ])
        // Databases
        .attach(model::ApiDatabase::fairing())
}

