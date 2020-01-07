#![feature(proc_macro_hygiene, decl_macro)]
use rocket::fairing::AdHoc;
use rocket::{get, routes, Rocket};
use rocket_jwt::{TokenSecret, JWT};

#[get("/")]
fn index(_jwt: JWT) -> String {
    "".to_owned()
}

fn rocket() -> Rocket {
    rocket::ignite()
        .mount("/", routes![index])
        .attach(AdHoc::on_attach("TokenSecret", |rocket| {
            let token_val = rocket.config().get_string("token_secret").unwrap();
            Ok(rocket.manage(TokenSecret(token_val)))
        }))
}

fn main() {
    let _ = rocket().launch();
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::encode;
    use jsonwebtoken::Header as jwtHeader;
    use rocket::http::{Header, Status};
    use rocket::local::Client;
    use rocket_jwt::Claims;

    fn jwt() -> String {
        let my_claims = Claims {
            exp: 10_000_000_000,
        };
        let key = "very_secret";
        encode(&jwtHeader::default(), &my_claims, key.as_ref()).unwrap()
    }

    #[rocket::async_test]
    async fn test_401() {
        let client = Client::new(rocket()).unwrap();
        let response = client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_200() {
        let header = Header::new("Authorization", jwt());

        let client = Client::new(rocket()).expect("valid rocket instance");
        let response = client.get("/").header(header).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}
