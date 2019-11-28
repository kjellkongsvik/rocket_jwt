#![feature(proc_macro_hygiene, decl_macro)]
use jsonwebtoken as jwt;
use rocket::{get, routes, Rocket};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    company: String,
    exp: usize,
}

#[get("/token")]
fn token() -> String {
    let claims = Claims {
        company: "ACME".to_string(),
        exp: 10000000000,
    };

    let token = jwt::encode(&jwt::Header::default(), &claims, "secret".as_ref()).unwrap();

    "Bearer ".to_string() + &token
}

fn parse_bearer(bearer_token: &str) -> Result<String, String> {
    let mut tk = bearer_token.split_whitespace();
    match tk.next() {
        Some(b) if b == "Bearer" => (),
        _ => return Err("Not a bearer".to_string()),
    }
    let token = tk.next().ok_or("missing token")?;
    if tk.next().is_some() {
        return Err("more than enough".to_string());
    }
    Ok(token.to_string())
}

fn rocket() -> Rocket {
    rocket::ignite().mount("/", routes![token])
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::parse_bearer;
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn missing_bearer() {
        assert_eq!(parse_bearer(""), Err("Not a bearer".to_string()));
    }

    #[test]
    fn missing_token() {
        assert_eq!(parse_bearer("Bearer "), Err("missing token".to_string()));
    }

    #[test]
    fn more_than_a_token() {
        assert_eq!(
            parse_bearer("Bearer a b"),
            Err("more than enough".to_string())
        );
    }

    #[test]
    fn parse_token() {
        assert_eq!(parse_bearer("Bearer a"), Ok("a".to_string()));
    }

    #[test]
    fn token() -> Result<(), String> {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let mut response = client.get("/token").dispatch();
        assert_eq!(response.status(), Status::Ok);

        let bearer_token = response.body_string().ok_or("error")?;
        let _token = parse_bearer(&bearer_token)?;
        Ok(())
    }
}
