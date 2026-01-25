# TimedMutes

A Rust-based service for managing timed mutes on Bluesky. It allows users to schedule mutes for specific durations and automatically resolves them using a cron scheduler.

## 🚀 Stack

- **Language:** [Rust](https://www.rust-lang.org/) (2021 Edition)
- **Framework:** [Actix-web](https://actix.rs/)
- **ORM:** [Diesel](https://diesel.rs/) (with SQLite)
- **Scheduler:** [tokio-cron-scheduler](https://github.com/m-cat/tokio-cron-scheduler)
- **Bluesky Integration:** `bsky-sdk`
- **Documentation:** [Utoipa](https://github.com/juhakivekas/utoipa) (Swagger UI)

## 📋 Requirements

- **Rust:** Latest stable version
- **SQLite:** Required for the database
- **Diesel CLI:** For running migrations (`cargo install diesel_cli --no-default-features --features sqlite`)

## 🛠 Setup & Installation

1.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd TimedMutes
    ```

2.  **Environment Configuration:**
    Create a `.env` file in the root directory and configure the necessary variables (see [Environment Variables](#-environment-variables)).
    ```bash
    echo "DATABASE_URL=test.db" > .env
    ```

3.  **Run Migrations:**
    ```bash
    diesel migration run
    ```

4.  **Build the project:**
    ```bash
    cargo build
    ```

## 🏃 Running the Application

### Locally
```bash
cargo run
```

### Docker
The project includes a `Dockerfile` for containerized deployment.
```bash
docker build -t timed-mutes .
docker run -p 9090:9090 --env-file .env timed-mutes
```

## ⚙️ Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `DATABASE_URL` | Path to the SQLite database file | **Required** |
| `SERVER_PORT` | Port for the HTTP server | `9090` |
| `CRON_ENABLED` | Enable the timed mute resolver scheduler (`1` to enable) | `0` |
| `CRON_SCHEDULE` | Cron expression for the scheduler | `0 1 * * * * *` |
| `ALLOWED_ORIGIN` | CORS allowed origin | `http://frontend.ripp.internal` |
| `DB_MIN_IDLE` | Minimum idle connections in the DB pool | `1` |
| `WORKER_COUNT` | Number of Actix-web workers | `2` |
| `COOKIE_DOMAIN` | Domain for session cookies | `.ripp.internal` |
| `HTTPS_ENABLED` | Use secure cookies (`1` for true) | `1` |

## 📖 API Documentation

The application provides an interactive Swagger UI for API exploration:
- **Swagger UI:** `http://localhost:9090/swagger-ui/`
- **OpenAPI Spec:** `http://localhost:9090/api-docs/openapi.json`

## 📂 Project Structure

- `src/main.rs`: Application entry point and server initialization.
- `src/tmute.rs`: Core logic for managing timed mutes and words.
- `src/user.rs`: Authentication and user-related handlers.
- `src/agent.rs`: Bluesky (Atproto) agent integration.
- `src/scheduler.rs`: Background task scheduling.
- `src/models.rs`: Diesel database models.
- `src/schema.rs`: Diesel database schema (auto-generated).
- `src/notification.rs`: Notification handling logic.
- `migrations/`: SQL migration files for database setup.
- `Dockerfile`: Multistage build configuration.

## 🧪 Tests

- **TODO:** Automated tests are currently not implemented. Contributions are welcome!
- To run tests (once added): `cargo test`

## 📜 License

- **TODO:** Specify license (e.g., MIT, Apache-2.0).
