# Usamos cargo-chef para optimizar el cache de dependencias
FROM lukemathwalker/cargo-chef:latest-rust-1.92.0-slim AS chef
WORKDIR /app

# Stage 1: Planner - Analiza el proyecto para determinar las dependencias
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Compila las dependencias (esta capa se cachea)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Instalar dependencias del sistema necesarias para compilar dependencias de Rust
# Usamos trixie (Debian Testing) como en el Dockerfile original si es necesario, 
# pero la imagen slim de cargo-chef suele estar basada en bookworm o bullseye.
# Para evitar conflictos de librerías, nos mantenemos consistentes.
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# "Cook" las dependencias. Si Cargo.toml/Cargo.lock no cambian, esto se recupera del cache de Docker.
RUN cargo chef cook --release --recipe-path recipe.json

# Ahora copiamos el código real y compilamos la aplicación
COPY . .
RUN cargo build --release --bin backend-axum-rust

# Stage 3: Runtime - Imagen final ligera para ejecución
FROM debian:trixie-slim AS runtime
WORKDIR /app

# Instalar dependencias de runtime
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar el binario desde el builder
COPY --from=builder /app/target/release/backend-axum-rust /app/backend-axum-rust

# Copiar recursos necesarios (migraciones, etc.)
COPY migrations /app/migrations

# Exponer puerto
EXPOSE 8000

# Comando de inicio
CMD ["./backend-axum-rust"]
