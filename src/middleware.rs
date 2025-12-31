use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
    http::{request::Parts, header},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{Claims, User},
};

pub struct CurrentUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    SqlitePool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extraer el token del header Authorization
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AppError::AuthError("Missing Authorization header".to_string()))?
            .to_str()
            .map_err(|_| AppError::AuthError("Invalid Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::AuthError("Invalid token format".to_string()));
        }

        let token = &auth_header[7..];

        // 2. Obtener el SECRET_KEY del entorno
        let secret = std::env::var("SECRET_KEY").unwrap_or_else(|_| "secret".to_string());

        // 3. Decodificar el token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::AuthError(format!("Invalid token: {}", e)))?;

        // 4. Obtener el usuario de la DB usando el state
        let pool = SqlitePool::from_ref(state);

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(token_data.claims.sub)
            .fetch_optional(&pool)
            .await
            .map_err(AppError::SqlxError)?;

        if let Some(user) = user {
            Ok(CurrentUser(user))
        } else {
            Err(AppError::AuthError("User not found".to_string()))
        }
    }
}
