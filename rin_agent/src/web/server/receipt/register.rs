use rocket::{
  Request, form::{Form, FromForm}, post, 
  response::{Responder,Result}, 
  Response,serde::{json::Json,Serialize}
};

#[derive(FromForm)]
struct RegistrationData {
    username: String,
    password: String,
    email: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct RegistrationResult {
    username: String,
    password: String,
    email: String,
}

impl<'r> Responder<'r, 'static> for RegistrationResult {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'static> {
        Response::build_from(Json(self).respond_to(req)?)
            .header(rocket::http::Header::new("X-Registration-Status", "Success"))
            .ok()
    }
}

#[post("/register", data = "<registration_data>")]
pub async fn register_receipt(
    registration_data: Form<RegistrationData>
) -> RegistrationResult {
  
  RegistrationResult {
    username: registration_data.username.clone(),
    password: registration_data.password.clone(),
    email: registration_data.email.clone(),
  }
}