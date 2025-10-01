use crate::Db;
use crate::psql_users::{Role, UserResponse, get_user};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize, uuid::Uuid};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub struct AuthError {
    msg: String,
}
impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for AuthError {}

pub fn create_token(username: String, jwt_secret: &[u8]) -> Result<String, AuthError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let claims = Claims {
        sub: username,
        iat: now,
        exp: now + (60 * 30), // 30 minutes expiration
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|e| AuthError { msg: e.to_string() })
}

async fn auth_by_role<'r>(req: &'r Request<'_>) -> Result<UserResponse, AuthError> {
    let jwt_secret = req.rocket().state::<Vec<u8>>().ok_or_else(|| AuthError {
        msg: "JWT Secret does not exist".to_string(),
    })?;
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
        &DecodingKey::from_secret(jwt_secret),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| AuthError { msg: e.to_string() })?;
    let db = req.rocket().state::<Db>().ok_or_else(|| AuthError {
        msg: "Database does not not exist".to_string(),
    })?;
    let calling_user = get_user(&token_data.claims.sub, &db.0)
        .await
        .map_err(|e| AuthError { msg: e.to_string() })?;
    Ok(calling_user)
}

pub struct Admin {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match auth_by_role(req).await {
            Ok(user) => user,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        if user.roles.iter().any(|user_role| match user_role {
            Role::Admin => true,
            _ => false,
        }) {
            Outcome::Success(Admin {
                id: user.id,
                username: user.username,
            })
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

pub struct Tutor {
    pub id: Uuid,
    #[allow(dead_code)]
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Tutor {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match auth_by_role(req).await {
            Ok(user) => user,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        if user.roles.iter().any(|user_role| match user_role {
            Role::Tutor => true,
            _ => false,
        }) {
            Outcome::Success(Tutor {
                id: user.id,
                username: user.username,
            })
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
pub struct Helper {
    pub id: Uuid,
    #[allow(dead_code)]
    pub username: String,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Helper {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match auth_by_role(req).await {
            Ok(user) => user,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        if user.roles.iter().any(|user_role| match user_role {
            Role::Helper => true,
            _ => false,
        }) {
            Outcome::Success(Helper {
                id: user.id,
                username: user.username,
            })
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match auth_by_role(req).await {
            Ok(user) => user,
            Err(e) => {
                println!("Error: {}", e);
                return Outcome::Error((Status::Unauthorized, ()));
            }
        };
        Outcome::Success(AuthenticatedUser {
            id: user.id,
            username: user.username,
        })
    }
}
/*
#[cfg(test)]
mod tests {
    use super::check_password;
    use super::hash_password;

    #[test]
    fn it_returns_ok_if_password_matches() {
        let result = hash_password("hello").unwrap();
        let result = check_password("hello", &result);
        assert!(result.is_ok());
    }
    #[test]
    fn it_returns_err_if_password_does_not_matche() {
        let result = hash_password("hello").unwrap();
        let result = check_password("hello2", &result);
        assert!(result.is_err());
    }
}
*/
