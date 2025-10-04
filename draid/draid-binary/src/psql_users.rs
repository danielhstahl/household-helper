use argon2::{
    Argon2,
    password_hash::{
        Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use futures::stream::StreamExt;
use rocket::serde::uuid::Uuid;
use rocket::serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Type, query, types::chrono};
use std::{collections::HashMap, fmt};
fn hash_password(password: &str) -> Result<String, Error> {
    // Generate a secure random salt
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}
fn check_password(password: &str, hashed_password: &str) -> Result<(), Error> {
    let parsed_hash = PasswordHash::new(&hashed_password)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
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
struct UserDB {
    id: Uuid,
    username: String,
}
#[derive(sqlx::FromRow)]
struct RoleDB {
    role: Role,
    username_id: Uuid,
}
#[derive(Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde")]
pub struct SessionDB {
    id: Uuid,
    username_id: Uuid,
    session_start: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub roles: Vec<Role>,
}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UserRequest<'a> {
    pub username: &'a str,
    pub password: Option<&'a str>,
    pub roles: Vec<Role>,
}

pub struct HashedPassword {
    hashed_password: String,
}

pub async fn authenticate_user(
    username: &str,
    password: &str,
    pool: &mut PgConnection,
) -> sqlx::Result<()> {
    let password_compare = sqlx::query_as!(
        HashedPassword,
        r#"
        SELECT hashed_password FROM users where username=$1
        "#,
        &username
    )
    .fetch_one(&mut *pool)
    .await?;

    check_password(&password, &password_compare.hashed_password)
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    Ok(())
}

pub async fn get_user(username: &str, pool: &mut PgConnection) -> sqlx::Result<UserResponse> {
    let user_db = sqlx::query_as!(
        UserDB,
        r#"
        SELECT id, username FROM users where username=$1
        "#,
        &username
    )
    .fetch_one(&mut *pool)
    .await?;
    let roles = sqlx::query_as!(
        RoleDB,
        r#"
        SELECT role as "role: Role", username_id FROM roles where username_id=$1
        "#,
        &user_db.id
    )
    .fetch_all(&mut *pool)
    .await?;
    Ok(UserResponse {
        id: user_db.id,
        username: user_db.username,
        roles: roles.into_iter().map(|v| v.role).collect(),
    })
}

pub async fn delete_user(username_id: &Uuid, pool: &mut PgConnection) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM roles where username_id=$1
        "#,
        &username_id
    )
    .execute(&mut *pool)
    .await?;
    sqlx::query!(
        r#"
        DELETE FROM users where id=$1
        "#,
        &username_id
    )
    .execute(&mut *pool)
    .await?;
    Ok(())
}

pub async fn get_all_users(pool: &mut PgConnection) -> sqlx::Result<Vec<UserResponse>> {
    let users_db = sqlx::query_as!(
        UserDB,
        r#"
        SELECT id, username FROM users
        "#,
    )
    .fetch_all(&mut *pool)
    .await?;
    let mut roles = sqlx::query_as!(
        RoleDB,
        r#"
        SELECT role as "role: Role", username_id FROM roles
        "#
    )
    .fetch(&mut *pool);
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

async fn create_roles(id: &Uuid, roles: &[Role], pool: &mut PgConnection) -> sqlx::Result<()> {
    let mut query_string = String::from("INSERT INTO roles (id, username_id, role) VALUES ");

    // Generate the multi-row `VALUES` placeholders
    // hilariously hacky...put the numbers in by dollar sign
    let role_placeholders: Vec<String> = (0..roles.len())
        .map(|i| format!("(gen_random_uuid(), ${}, ${})", 2 * i + 1, 2 * i + 2))
        .collect();

    query_string.push_str(&role_placeholders.join(", "));

    // Create a `Query` object with the dynamic SQL string
    let mut sqlx_query = query(&query_string);

    // Bind each value individually to the query, including the enum
    for role in roles {
        sqlx_query = sqlx_query.bind(&id).bind(role);
    }
    sqlx_query.execute(pool).await?;
    Ok(())
}
pub async fn create_user<'a>(user: &UserRequest<'a>, pool: &mut PgConnection) -> sqlx::Result<()> {
    let password = user
        .password
        .ok_or_else(|| sqlx::Error::Protocol("Password is required to create user".to_string()))?;
    let hashed_password =
        hash_password(&password).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
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
    .fetch_one(&mut *pool)
    .await?;
    create_roles(&user_db.id, &user.roles, pool).await?;
    Ok(())
}

pub async fn patch_user<'a>(
    id: &Uuid,
    user: &UserRequest<'a>,
    pool: &mut PgConnection,
) -> sqlx::Result<()> {
    match &user.password {
        Some(password) => {
            let hashed_password =
                hash_password(password).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
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
            .execute(&mut *pool)
        }
        None => sqlx::query_as!(
            UserDB,
            r#"
                UPDATE users
                set username=$1
                where id=$2
                "#,
            &user.username,
            &id
        )
        .execute(&mut *pool),
    }
    .await?;

    sqlx::query_as!(
        UserDB,
        r#"
        DELETE FROM roles
        WHERE username_id=$1
        "#,
        &id
    )
    .execute(&mut *pool)
    .await?;
    create_roles(&id, &user.roles, pool).await?;
    Ok(())
}

pub async fn create_session<'a>(
    username_id: &Uuid,
    pool: &mut PgConnection,
) -> sqlx::Result<SessionDB> {
    let session_db = sqlx::query_as!(
        SessionDB,
        r#"
        INSERT INTO sessions (id, username_id, session_start)
        VALUES (gen_random_uuid(), $1, NOW())
        RETURNING id, username_id, session_start
        "#,
        &username_id
    )
    .fetch_one(pool)
    .await?;

    Ok(session_db)
}

pub async fn get_all_sessions<'a>(
    username_id: &Uuid,
    pool: &mut PgConnection,
) -> sqlx::Result<Vec<SessionDB>> {
    let session_db = sqlx::query_as!(
        SessionDB,
        r#"
        SELECT id, username_id, session_start
        from sessions WHERE username_id=$1
        ORDER BY session_start DESC
        "#,
        &username_id
    )
    .fetch_all(pool)
    .await?;

    Ok(session_db)
}

pub async fn get_most_recent_session<'a>(
    username_id: &Uuid,
    pool: &mut PgConnection,
) -> sqlx::Result<Option<SessionDB>> {
    let session_db = sqlx::query_as!(
        SessionDB,
        r#"
        SELECT id, username_id, session_start
        from sessions WHERE username_id=$1
        ORDER BY session_start DESC
        "#,
        &username_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(session_db)
}

pub async fn delete_session<'a>(
    session_id: &Uuid,
    user_id: &Uuid,
    pool: &mut PgConnection,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM sessions WHERE id=$1
        AND username_id=$2
        "#,
        &session_id,
        &user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

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
