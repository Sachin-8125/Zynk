use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, AppState};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

fn create_token(user_id: Uuid, secret: &str) -> Result<String, AppError> {
    let claims = Claims {
        sub: user_id,
        exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal)
}

pub struct AuthUser(pub Uuid);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let token = header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;
        Ok(AuthUser(data.claims.sub))
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserOut {
    id: Uuid,
    username: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserOut,
}

#[derive(Deserialize)]
pub struct RegisterInput {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    email: String,
    password: String,
}

pub async fn register(
    State(s): State<AppState>,
    Json(i): Json<RegisterInput>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let username = i.username.trim().to_string();
    let email = i.email.trim().to_lowercase();
    if !(3..=30).contains(&username.chars().count()) {
        return Err(AppError::BadRequest("Username must be 3 to 30 characters".into()));
    }
    if !email.contains('@') {
        return Err(AppError::BadRequest("Enter a valid email address".into()));
    }
    if i.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(i.password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal)?
        .to_string();

    let user: UserOut = sqlx::query_as(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username",
    )
    .bind(&username)
    .bind(&email)
    .bind(&hash)
    .fetch_one(&s.db)
    .await?;

    let token = create_token(user.id, &s.jwt_secret)?;
    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginInput>,
) -> Result<Json<AuthResponse>, AppError> {
    let row: Option<(Uuid, String, String)> =
        sqlx::query_as("SELECT id, username, password_hash FROM users WHERE email = $1")
            .bind(payload.email.trim().to_lowercase())
            .fetch_optional(&state.db)
            .await?;
    let (id, username, hash) = row.ok_or(AppError::BadRequest("Wrong email or password".into()))?;

    let parsed = PasswordHash::new(&hash).map_err(|_| AppError::Internal)?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed)
        .map_err(|_| AppError::BadRequest("Wrong email or password".into()))?;

    let token = create_token(id, &state.jwt_secret)?;
    Ok(Json(AuthResponse { token, user: UserOut { id, username } }))
}

pub async fn me(State(s): State<AppState>, AuthUser(uid): AuthUser) -> Result<Json<UserOut>, AppError> {
    let user = sqlx::query_as("SELECT id, username FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&s.db)
        .await?;
    Ok(Json(user))
}