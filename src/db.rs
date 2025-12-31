use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::str::FromStr;
use std::fs::File;
use std::path::Path;

pub async fn establish_connection(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Verificar si el archivo de base de datos existe, si no, crearlo.
    let db_path = database_url.strip_prefix("sqlite://").unwrap_or("data.db");
    if !Path::new(db_path).exists() {
        println!("Database file not found, creating: {}", db_path);
        File::create(db_path).expect("Failed to create database file");
    }

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Ejecutar migraciones
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    println!("Migrations executed successfully");

    Ok(pool)
}
