# Builder stage
FROM rust:1.92.0-slim-trixie as builder

WORKDIR /usr/src/app
COPY . .

# Necesario para compilar dependencias native-tls/sqlite si no vienen estáticas
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Compilar en modo release
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim

WORKDIR /app

# Instalar dependencias de runtime (openssl, ca-certificates, sqlite3 lib)
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar el binario desde el builder
COPY --from=builder /usr/src/app/target/release/backend-axum-rust /app/backend-axum-rust

# Copiar migraciones y .env (opcional, mejor inyectar ENV vars en deploy)
COPY migrations /app/migrations
# COPY .env /app/.env # En producción, usa variables de entorno reales, no .env file

# Exponer puerto
EXPOSE 8000

# Comando de inicio
CMD ["./backend-axum-rust"]
