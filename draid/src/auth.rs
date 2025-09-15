use crate::Db;
use crate::psql_users::Role;
use crate::psql_users::get_user;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use rocket::http::{Header, Status};
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize, json::Json};
use std::fmt;
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

#[derive(Debug)]
struct AuthError {
    msg: String,
}
impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for AuthError {}

async fn auth_by_role<'r>(req: &'r Request<'_>) -> Result<Vec<Role>, AuthError> {
    let auth_header = req.headers().get_one("Authorization");

    let header = auth_header.ok_or_else(|| AuthError {
        msg: "Header does not not exist".to_string(),
    })?;
    let token = if header.starts_with("Bearer ") {
        Ok(&header[7..]) // Get the token part after "Bearer "
    } else {
        Err(AuthError {
            msg: "Header does not start with Bearer".to_string(),
        })
    }?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::new(Algorithm::RS256),
    )
    .map_err(|e| AuthError { msg: e.to_string() })?;

    let db = req.rocket().state::<Db>().ok_or_else(|| AuthError {
        msg: "Database does not not exist".to_string(),
    })?;

    let calling_user = get_user(&token_data.claims.sub, &db.0)
        .await
        .map_err(|e| AuthError { msg: e.to_string() })?;
    Ok(calling_user.roles)
}

pub struct Admin;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let roles = match auth_by_role(req).await {
            Ok(roles) => roles,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        if roles.iter().any(|user_role| match user_role {
            Role::Admin => true,
            _ => false,
        }) {
            Outcome::Success(Admin {})
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

pub struct Tutor;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Tutor {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let roles = match auth_by_role(req).await {
            Ok(roles) => roles,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        if roles.iter().any(|user_role| match user_role {
            Role::Tutor => true,
            _ => false,
        }) {
            Outcome::Success(Tutor {})
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
pub struct Helper;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Helper {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let roles = match auth_by_role(req).await {
            Ok(roles) => roles,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        if roles.iter().any(|user_role| match user_role {
            Role::Helper => true,
            _ => false,
        }) {
            Outcome::Success(Helper {})
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

pub struct AuthenticatedUser;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if auth_by_role(req).await.is_err() {
            return Outcome::Error((Status::Unauthorized, ()));
        };
        Outcome::Success(AuthenticatedUser {})
    }
}
