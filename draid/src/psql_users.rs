use argon2::{
    Argon2,
    password_hash::{Error, PasswordHasher, SaltString, rand_core::OsRng},
};
use rocket::serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Type, query, types::Uuid};

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
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserResponse {
    id: String, //uuid but serde doesn't recognize it
    username: String,
    roles: Vec<Role>,
}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserRequest<'a> {
    //id: String, //uuid but serde doesn't recognize it
    username: &'a str,
    password: &'a str,
    roles: Vec<Role>,
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
        SELECT id, role as "role: Role" FROM roles where username_id=$1
        "#,
        &user_db.id
    )
    .fetch_all(pool)
    .await?;
    Ok(UserResponse {
        id: user_db.id.to_string(),
        username: user_db.username,
        roles: roles.into_iter().map(|v| v.role).collect(),
    })
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
