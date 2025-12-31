# Backend Axum Rust

Este proyecto es una reescritura del backend de FastAPI utilizando Rust y el framework Axum. Provee una API para gestión de tareas con autenticación JWT y persistencia en SQLite.

## Requisitos

- Rust (latest stable)
- SQLite (opcional, sqlx crea el archivo si no existe, pero `sqlite3` CLI es útil para debug)

## Configuración Local

1.  **Variables de Entorno**:
    El proyecto usa un archivo `.env`. Se incluye uno por defecto:

    ```bash
    DATABASE_URL=sqlite://data.db
    SECRET_KEY=supersecretkeyformigbertbackend
    RUST_LOG=debug
    ```

2.  **Base de Datos**:
    Al iniciar el servidor, el sistema intentará crear (`migrations`) automáticamente la base de datos `data.db` y las tablas necesarias.

## Ejecución

Para correr el servidor en modo desarrollo:

```bash
cargo run
```

El servidor escuchará en `http://localhost:8000`.

## Documentación de la API (Swagger UI)

El proyecto incluye una interfaz interactiva para explorar y probar la API.

1.  **Acceso**: Con el servidor corriendo, visita: [http://localhost:8000/swagger-ui](http://localhost:8000/swagger-ui)
2.  **Uso de Autenticación**:
    - Obtén un token usando el endpoint `/token`.
    - Haz clic en el botón **"Authorize"** en la parte superior derecha de la página de Swagger.
    - Pega el token y confirma.
    - ¡Ya puedes probar los endpoints de tareas directamente!

## Pruebas Manuales (cURL)

### 1. Registrar Usuario

```bash
curl -X POST http://localhost:8000/users/ \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}'
```

### 2. Login (Obtener Token)

```bash
curl -X POST http://localhost:8000/token \
  -H "Content-Type: application/json" \
  -d '{"username": "test@example.com", "password": "password123"}'
```

Copiar el `access_token` de la respuesta.

### 3. Crear Tarea

```bash
TOKEN="TU_ACCESS_TOKEN_AQUI"
curl -X POST http://localhost:8000/tasks/ \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Aprender Axum", "description": "Backend en Rust", "completed": false}'
```

### 4. Listar Tareas

```bash
curl -X GET http://localhost:8000/tasks/ \
  -H "Authorization: Bearer $TOKEN"
```

## Despliegue (Deployment)

Para desplegar en un servidor, se recomienda usar Docker.

### Dockerfile

Construir la imagen:

```bash
docker build -t backend-axum-rust .
```

Correr el contenedor:

```bash
docker run -d -p 8000:8000 -v $(pwd)/data:/app/data --name backend-rust backend-axum-rust
```

_Nota: Se monta un volumen para persistir la base de datos SQLite._

### Compilación Manual (Release)

Si deseas correr el binario directamente en el servidor Linux:

```bash
cargo build --release
./target/release/backend-axum-rust
```

Asegúrate de tener el archivo `.env` y la carpeta `migrations` en el mismo directorio de ejecución (o configurar las rutas adecuadamente).
