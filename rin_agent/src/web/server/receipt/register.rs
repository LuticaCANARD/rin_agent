use rocket::{
  Request, Response, form::{Form, FromForm}, http::Header, post, response::{Responder,Result}, serde::{Serialize, json::Json}
};

#[derive(FromForm)]
pub struct RegistrationData {
    username: String,
    password: String,
    email: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegistrationResult {
    success: bool,
    message: String,
    #[serde(skip)]
    pub custom_header: Option<String>,
}

impl<'r> Responder<'r, 'static> for RegistrationResult {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'static> {
        let custom_header = self.custom_header.clone();
        let mut builder = Response::build_from(Json(self).respond_to(req)?);

        if let Some(value) = custom_header {
            builder.header(rocket::http::Header::new("X-Custom-Header", value));
        }

        builder.header(rocket::http::Header::new("X-Registration-Status", "Success"))
            .ok()
    }
}

pub struct RequestHeaders<'r>(pub &'r rocket::http::HeaderMap<'r>);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for RequestHeaders<'r> {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        rocket::request::Outcome::Success(RequestHeaders(req.headers()))
    }
}

#[post("/register", data = "<registration_data>")]
pub async fn register_receipt(
  headers: RequestHeaders<'_>,
  registration_data: Form<RegistrationData>
) -> RegistrationResult {
    // 예시: 특정 헤더 가져오기
    let user_agent = headers.0.get_one("User-Agent");
    
    // Here you would typically handle the registration logic,
    // such as saving the data to a database.
    
    // For demonstration, we will just return a success message.
    RegistrationResult {
        success: true,
        message: format!(
            "User '{}' registered successfully with email '{}'.",
            registration_data.username, registration_data.email
        ),
        custom_header: Some("Dynamic-Value-From-Handler".to_string()),
    }
}