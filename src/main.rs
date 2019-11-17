#![feature(proc_macro_hygiene, decl_macro)]
use jsonwebtoken as jwt;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{get, routes, Outcome, Rocket};
use serde_derive::{Deserialize, Serialize};

struct ApiKey(String);

fn is_valid(key: &str) -> bool {
    let mut b = key.split_whitespace();
    match b.next() {
        Some(s) if s == "bearer" => (),
        _ => return false,
    };
    let token = match b.next() {
        Some(t) => {
            println!("{}", t);

            match jwt::decode::<Claims>(t, "secret".as_ref(), &jwt::Validation::default()) {
                Ok(c) => Some(c),
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
        _ => None,
    };
    match token {
        Some(_) => true,
        _ => false,
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    company: String,
    exp: usize,
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
    rocket::ignite().mount("/", routes![index])
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::{rocket, Claims};
    use jsonwebtoken as jwt;
    use rocket::http::{Header, Status};
    use rocket::local::Client;

    #[test]
    fn hello() {
        let claims = Claims {
            company: "ACME".to_owned(),
            exp: 10000000000,
        };

        let token = jwt::encode(&jwt::Header::default(), &claims, "secret".as_ref()).unwrap();

        let header = Header::new("Authentication", "bearer ".to_owned() + &token);
        let client = Client::new(rocket()).expect("valid rocket instance");
        let response = client.get("/").header(header).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
