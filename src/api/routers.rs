use rocket::{post,get};

use std::sync::LazyLock;


#[get("/")]
pub async fn test_index() -> &'static str {
    "Hello, world!"
}

#[catch(404)]
pub fn not_found() -> &'static str {
    "404 Not Found"
}



pub fn get_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![test_index])
        .register("/", catchers![not_found])
}