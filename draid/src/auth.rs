use crate::Db;
use crate::psql_users::get_user;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use rocket::http::{Header, Status};
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize, json::Json};
use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};
pub const JWT_SECRET: &[u8] = b"secret-jwt-key";

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    // The subject of the token (the user's ID or username)
    pub sub: String,
    // Issued at (as a timestamp)
    pub iat: u64,
    // Expiration time (as a timestamp)
    pub exp: u64,
}
pub struct AuthToken(pub Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthToken {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = req.headers().get_one("Authorization");

        let token = if let Some(header) = auth_header {
            if header.starts_with("Bearer ") {
                &header[7..] // Get the token part after "Bearer "
            } else {
                return Outcome::Error((Status::Unauthorized, ()));
            }
        } else {
            return Outcome::Error((Status::Unauthorized, ()));
        };

        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::new(Algorithm::RS256),
        ) {
            Ok(token) => token,
            Err(e) => return Outcome::Error((Status::Unauthorized, ())),
        };

        let db = match req.rocket().state::<Db>() {
            Some(db) => db,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };
        let calling_user = match get_user(&token_data.claims.sub, &db.0).await {
            Ok(user) => user,
            Err(e) => return Outcome::Error((Status::Unauthorized, ())),
        };
        Outcome::Success(AuthToken(token_data.claims))
        //Outcome::Success(calling_user)
        /*if token_data.sub{

        }*/
        /*match token_data {
            Ok(data) => Success(AuthToken(data.claims)),
            Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }*/
    }
}
