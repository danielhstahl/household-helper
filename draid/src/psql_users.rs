use argon2::{
    Argon2,
    password_hash::{Error, PasswordHasher, SaltString, rand_core::OsRng},
};
use futures::stream::StreamExt;
use rocket::serde::uuid::Uuid;
use rocket::serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Type, query};
use std::{collections::HashMap, fmt};
fn hash_password(password: &str) -> Result<String, Error> {
    // Generate a secure random salt
    let salt = SaltString::generate(&mut OsRng);

    // Create an Argon2 instance with default parameters
    let argon2 = Argon2::default();

    // Hash the password using Argon2
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

#[derive(Serialize, Deserialize, Type)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
#[sqlx(type_name = "role_type", rename_all = "lowercase")]
pub enum Role {
    Admin,
    Helper,
    Tutor,
}

//only for printing a nice error in auth.rs
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Helper => write!(f, "helper"),
            Role::Tutor => write!(f, "tutor"),
        }
    }
}

#[derive(sqlx::FromRow)]
//#[serde(crate = "rocket::serde")]
struct UserDB {
    id: Uuid,
    username: String,
}
#[derive(sqlx::FromRow)]
//#[serde(crate = "rocket::serde")]
struct RoleDB {
    id: Uuid,
    role: Role,
    username_id: Uuid,
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserResponse {
    id: Uuid, //uuid but serde doesn't recognize it
    username: String,
    pub roles: Vec<Role>,
}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserRequest<'a> {
    //id: String, //uuid but serde doesn't recognize it
    username: &'a str,
    password: &'a str,
    roles: Vec<Role>,
}

pub struct HashedPassword {
    hashed_password: String,
}
pub async fn authenticate_user(
    username: &str,
    password: &str,
    pool: &Pool<Postgres>,
) -> sqlx::Result<()> {
    let password_compare = sqlx::query_as!(
        HashedPassword,
        r#"
        SELECT hashed_password FROM users where username=$1
        "#,
        &username
    )
    .fetch_one(pool)
    .await?;
    let hashed_password =
        hash_password(&password).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    if password_compare.hashed_password == hashed_password {
        ()
    }

    Err(sqlx::Error::Protocol("Unauthorized".to_string()))
}

pub async fn get_user(username: &str, pool: &Pool<Postgres>) -> sqlx::Result<UserResponse> {
    //todo do this in parallel
    let user_db = sqlx::query_as!(
        UserDB,
        r#"
        SELECT id, username FROM users where username=$1
        "#,
        &username
    )
    .fetch_one(pool)
    .await?;
    let roles = sqlx::query_as!(
        RoleDB,
        r#"
        SELECT id, role as "role: Role", username_id FROM roles where username_id=$1
        "#,
        &user_db.id
    )
    .fetch_all(pool)
    .await?;
    Ok(UserResponse {
        id: user_db.id,
        username: user_db.username,
        roles: roles.into_iter().map(|v| v.role).collect(),
    })
}

pub async fn delete_user(username_id: &Uuid, pool: &Pool<Postgres>) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM roles where username_id=$1
        "#,
        &username_id
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        r#"
        DELETE FROM users where id=$1
        "#,
        &username_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_users(pool: &Pool<Postgres>) -> sqlx::Result<Vec<UserResponse>> {
    let users_db = sqlx::query_as!(
        UserDB,
        r#"
        SELECT id, username FROM users
        "#,
    )
    .fetch_all(pool)
    .await?;
    let mut roles = sqlx::query_as!(
        RoleDB,
        r#"
        SELECT id, role as "role: Role", username_id FROM roles
        "#
    )
    .fetch(pool);
    let mut roles_by_user: HashMap<Uuid, Vec<Role>> = HashMap::new();
    while let Some(role) = roles.next().await {
        let role = role?;
        roles_by_user
            .entry(role.username_id)
            .or_insert_with(Vec::new)
            .push(role.role);
    }
    Ok(users_db
        .into_iter()
        .map(|user| UserResponse {
            id: user.id,
            username: user.username,
            roles: roles_by_user.remove(&user.id).unwrap_or_default(),
        })
        .collect())
}

pub async fn create_user<'a>(user: &UserRequest<'a>, pool: &Pool<Postgres>) -> sqlx::Result<()> {
    let hashed_password =
        hash_password(&user.password).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let user_db = sqlx::query_as!(
        UserDB,
        r#"
        INSERT INTO users (id, username, hashed_password)
        VALUES (gen_random_uuid(), $1, $2)
        RETURNING id, username
        "#,
        &user.username,
        &hashed_password
    )
    .fetch_one(pool)
    .await?;

    let mut query_string = String::from("INSERT INTO roles (id, username_id, role) VALUES ");

    // Generate the multi-row `VALUES` placeholders
    // hilariously hacky...put the numbers in by dollar sign
    let role_placeholders: Vec<String> = (0..user.roles.len())
        .map(|i| format!("(gen_random_uuid(), ${}, ${})", 2 * i + 1, 2 * i + 2))
        .collect();

    query_string.push_str(&role_placeholders.join(", "));

    // Create a `Query` object with the dynamic SQL string
    let mut sqlx_query = query(&query_string);

    // Bind each value individually to the query, including the enum
    for role in &user.roles {
        sqlx_query = sqlx_query.bind(&user_db.id).bind(role);
    }
    sqlx_query.execute(pool).await?;
    Ok(())
}

pub async fn patch_user<'a>(
    id: &Uuid,
    user: &UserRequest<'a>,
    pool: &Pool<Postgres>,
) -> sqlx::Result<()> {
    let hashed_password =
        hash_password(&user.password).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    sqlx::query_as!(
        UserDB,
        r#"
        UPDATE users
        set username=$1, hashed_password=$2
        where id=$3
        "#,
        &user.username,
        &hashed_password,
        &id
    )
    .execute(pool)
    .await?;
    sqlx::query_as!(
        UserDB,
        r#"
        DELETE FROM roles
        WHERE username_id=$1
        "#,
        &id
    )
    .execute(pool)
    .await?;

    let mut query_string = String::from("INSERT INTO roles (id, username_id, role) VALUES ");

    // Generate the multi-row `VALUES` placeholders
    // hilariously hacky...put the numbers in by dollar sign
    let role_placeholders: Vec<String> = (0..user.roles.len())
        .map(|i| format!("(gen_random_uuid(), ${}, ${})", 2 * i + 1, 2 * i + 2))
        .collect();

    query_string.push_str(&role_placeholders.join(", "));

    // Create a `Query` object with the dynamic SQL string
    let mut sqlx_query = query(&query_string);

    // Bind each value individually to the query, including the enum
    for role in &user.roles {
        sqlx_query = sqlx_query.bind(&id).bind(role);
    }
    sqlx_query.execute(pool).await?;
    Ok(())
}
