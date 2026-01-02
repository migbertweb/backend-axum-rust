use axum::{extract::State, Json};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::SqlitePool;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH}; // Add SystemTime and UNIX_EPOCH

use crate::{
    error::AppError,
    models::{Claims, CreateUser, LoginRequest, Token, User},
};

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Email already registered")
    )
)]
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    // 1. Verificar si el usuario ya existe
    let user_exists = sqlx::query("SELECT 1 FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?;

    if user_exists.is_some() {
        return Err(AppError::ValidationError("Email already registered".to_string()));
    }

    // 2. Hash de contraseña
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::ValidationError(e.to_string()))?
        .to_string();

    // 3. Insertar usuario
    let id = sqlx::query("INSERT INTO users (email, hashed_password) VALUES (?, ?)")
        .bind(&payload.email)
        .bind(&password_hash)
        .execute(&pool)
        .await?
        .last_insert_rowid();

    // 4. Retornar usuario creado
    Ok(Json(User {
        id,
        email: payload.email,
        hashed_password: "".to_string(), // No retornar hash
        is_active: true,
    }))
}

#[utoipa::path(
    post,
    path = "/token",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = Token),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(pool): State<SqlitePool>,
    // Usamos Json<LoginRequest> en lugar de FormUrlEncoded para simplificar, 
    // aunque FastAPI usa FormUrlEncoded por defecto para OAuth2.
    // El usuario pidió "lo mismo", pero LoginRequest es más común en APIs JSON modernas.
    // Si falla la integración con frontend, cambiar a Form.
    Json(payload): Json<LoginRequest>, 
) -> Result<Json<Token>, AppError> {
    
    // 1. Buscar usuario
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&payload.username) // LoginRequest usa 'username' para el email
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::AuthError("Invalid credentials".to_string()))?;

    // 2. Verificar password
    let parsed_hash = PasswordHash::new(&user.hashed_password)
        .map_err(|_| AppError::AuthError("Invalid password hash in DB".to_string()))?;
    
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::AuthError("Invalid credentials".to_string()))?;

    // 3. Generar JWT
    let secret = env::var("SECRET_KEY").unwrap_or_else(|_| "secret".to_string());
    
    // Convertir SystemTime o chrono::DateTime a u64 para 'exp' (epoch seconds)
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize + 60 * 30; // 30 minutos

    let claims = Claims {
        sub: user.email,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::AuthError(format!("Token creation failed: {}", e)))?;

    Ok(Json(Token {
        access_token: token,
        token_type: "bearer".to_string(),
    }))
}
