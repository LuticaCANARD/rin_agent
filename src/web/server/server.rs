use rocket::{get, post, Config};
use super::super::api::statius::get_status;

use std::sync::LazyLock;


#[get("/")]
pub async fn test_index() -> &'static str {
    "Hello, world!"
}

#[catch(404)]
pub fn not_found() -> &'static str {
    "404 Not Found"
}

#[get("/query")]
pub fn test_query() -> &'static str {
    "----------------"
}

pub fn get_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![
            test_index,
            get_status,
            test_query
        ])
        .register("/", catchers![not_found])
        
        
}