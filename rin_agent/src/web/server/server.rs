use rocket::catch;
use rocket::get;
use rocket::routes;
use rocket::catchers;

use crate::web::server::receipt::register::register_receipt;
use super::super::api::status::get_status;

#[get("/")]
pub async fn test_index() -> &'static str {
    "Hello, this is Rin Agent API"
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
        .mount("/", rocket::fs::FileServer::from("./client"))
        .mount("/api/", routes![
            test_index,
            get_status,
            test_query,
            register_receipt
        ])
        .register("/", catchers![not_found])
}