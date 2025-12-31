use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;
use utoipa::ToSchema;

// --- Domain Models (Mapped to DB) ---

#[derive(Debug, Serialize, FromRow, Clone, ToSchema)]
pub struct User {
    pub id: i64, // SQLite uses INTEGER which maps to i64
    pub email: String,
    #[serde(skip)] // No serializar el hash en la respuesta JSON
    pub hashed_password: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: NaiveDateTime,
    pub owner_id: i64,
}

// --- Request/Response DTOs ---

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String, // FastAPI OAuth2PasswordRequestForm usa 'username' para el email
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

// Claims para JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Email del usuario
    pub exp: usize,
}
