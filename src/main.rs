use axum::{
    routing::{get, post, put, delete},
    Router,
};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, Modify};
use utoipa_swagger_ui::SwaggerUi;

mod db;
mod error;
mod handlers;
mod middleware;

mod models;

#[cfg(test)]
mod tests;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register,
        handlers::auth::login,
        handlers::tasks::create_task,
        handlers::tasks::get_tasks,
        handlers::tasks::get_task,
        handlers::tasks::update_task,
        handlers::tasks::delete_task
    ),
    components(
        schemas(
            models::User, 
            models::CreateUser, 
            models::LoginRequest, 
            models::Token, 
            models::Task, 
            models::CreateTask, 
            models::UpdateTask, 
            handlers::tasks::Pagination
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "tasks", description = "Task management endpoints")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cargar variables de entorno
    dotenv().ok();

    // Inicializar tracing (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info,backend_axum_rust=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Conectar a base de datos
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::establish_connection(&database_url).await?;

    // Crear app
    let app = create_app(pool);

    // Iniciar servidor
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_app(pool: sqlx::SqlitePool) -> Router {
    // Configurar CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Rutas públicas
        .route("/", get(|| async { "Axum Backend is running!" }))
        .route("/users", post(handlers::auth::register))
        .route("/token", post(handlers::auth::login))
        // Rutas protegidas
        .route("/tasks", post(handlers::tasks::create_task))
        .route("/tasks", get(handlers::tasks::get_tasks))
        .route("/tasks/:id", get(handlers::tasks::get_task))
        .route("/tasks/:id", put(handlers::tasks::update_task))
        .route("/tasks/:id", delete(handlers::tasks::delete_task))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(pool)
}
