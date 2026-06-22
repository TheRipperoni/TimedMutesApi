use crate::models::TimedMute;
use crate::models::TimedMuteWord;
use crate::tmute::CreateTimedMuteRequest;
use crate::tmute::DeleteTimedMuteRequest;
use crate::user::IsActiveSuccessResponse;
use crate::user::LoginRequest;
use std::env;

use crate::scheduler::start_scheduler;
use crate::tmute::{create, create_word, delete, delete_word, list, list_word, trigger};
use crate::user::{is_active, login, logout};
use axum::{
    routing::{get, post},
    Router,
};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use dotenvy::dotenv;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use axum::http::{header, Method};

pub mod agent;
pub mod error;
pub mod helper;
pub mod models;
mod scheduler;
pub mod schema;
mod tmute;
mod user;

pub const APPLICATION_JSON: &str = "application/json";

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub const USER_ID_KEY: &str = "user_id";
pub const USER_HANDLE_KEY: &str = "user_handle";
pub const DID_KEY: &str = "did";
pub const ACTIVE_KEY: &str = "active";
pub const ACCESS_JWT_KEY: &str = "access_jwt";
pub const REFRESH_JWT_KEY: &str = "refresh_jwt";
pub const COOKIE_DATE_KEY: &str = "cookie_date";

#[derive(OpenApi)]
#[openapi(
    paths(
        tmute::create,
        tmute::list,
        tmute::delete,
        tmute::list_word,
        tmute::create_word,
        tmute::delete_word,
        user::login,
        user::logout,
        user::is_active
    ),
    components(schemas(
        TimedMute,
        TimedMuteWord,
        CreateTimedMuteRequest,
        LoginRequest,
        DeleteTimedMuteRequest,
        IsActiveSuccessResponse,
    ))
)]
struct ApiDoc;

fn init_db(database_url: &str, db_min_idle: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .min_idle(Some(db_min_idle.parse().unwrap()))
        .build(manager)
        .expect("Failed to create pool")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env::set_var("RUST_LOG", "axum=debug");
    env_logger::init();

    // Get Environment Variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let db_min_idle = env::var("DB_MIN_IDLE").unwrap_or("1".to_string());
    let cron_schedule = env::var("CRON_SCHEDULE").unwrap_or("0 1 * * * * *".to_string());
    let allowed_origin =
        env::var("ALLOWED_ORIGIN").unwrap_or("http://frontend.ripp.internal".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("9090".to_string());
    let cron_enabled = env::var("CRON_ENABLED").unwrap_or("0".to_string()) == "1";

    // Create DB Pool
    let db_pool = init_db(database_url.as_str(), db_min_idle.as_str());

    // Start Scheduler
    if cron_enabled {
        start_scheduler(cron_schedule.as_str()).await;
    }

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin.parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCESS_CONTROL_ALLOW_ORIGIN])
        .allow_credentials(true);

    // Sessions
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(env::var("HTTPS_ENABLED").unwrap_or("1".to_string()) == "1")
        .with_same_site(SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::weeks(1)));

    // Router
    let app = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/timed-mutes", get(list))
        .route("/timed-mute", post(create))
        .route("/deleteTimedMute", post(delete))
        .route("/trigger", post(trigger))
        .route("/active", get(is_active))
        .route("/timed-mute-words", get(list_word))
        .route("/timed-mute-word", post(create_word))
        .route("/deleteTimedMuteWord", post(delete_word))
        .route("/deactivate", post(crate::user::deactivate))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(session_layer)
        .layer(cors)
        .with_state(db_pool);

    // Start Http Server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", server_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
