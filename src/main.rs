#![feature(proc_macro_hygiene, decl_macro)]
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket::State;

#[macro_use]
extern crate rocket;

struct ApiKey(String);

struct MyConfig {
    user_val: String,
}

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    key == "valid_api_key"
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
        let u = request.guard::<State<MyConfig>>().unwrap();

        println!("{:?}", u.user_val);
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => Outcome::Success(ApiKey(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}

#[get("/")]
fn index(key: ApiKey, state: State<MyConfig>) -> &'static str {
    "Hello, world!\n"
}

fn main() {
    let config = MyConfig {
        user_val: "user input".to_string(),
    };
    rocket::ignite()
        .mount("/", routes![index])
        .manage(config)
        .launch();
}
