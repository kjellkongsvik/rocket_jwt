#![feature(proc_macro_hygiene, decl_macro)]
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Outcome, Rocket, State};

#[macro_use]
extern crate dotenv_codegen;

#[macro_use]
extern crate rocket;

struct ApiKey(String);

struct JWKS {
    user_val: String,
}

fn is_valid(key: &str) -> bool {
    key == "bearer "
}

#[derive(Debug)]
enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let u = request.guard::<State<JWKS>>().unwrap();

        println!("{:?}", u.user_val);
        let keys: Vec<_> = request.headers().get("Authentication").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => Outcome::Success(ApiKey(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}

#[get("/")]
fn index(_key: ApiKey) -> &'static str {
    "Hello, world!\n"
}

fn rocket() -> Rocket {
    let config = JWKS {
        user_val: dotenv!["AUTHSERVER"].to_string(),
    };
    rocket::ignite().mount("/", routes![index]).manage(config)
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::{Header, Status};
    use rocket::local::Client;

    #[test]
    fn index() {
        let header = Header::new("Authentication", "bearer ");
        let client = Client::new(rocket()).expect("valid rocket instance");
        let response = client.get("/").header(header).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
