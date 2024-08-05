use crate::models::TimedMute;
use crate::models::TimedMuteWord;
use crate::tmute::CreateTimedMuteRequest;
use crate::tmute::DeleteTimedMuteRequest;
use crate::user::IsActiveSuccessResponse;
use crate::user::LoginRequest;
use std::{env, io};

use crate::scheduler::start_scheduler;
use crate::tmute::{create, create_word, delete, delete_word, list, list_word, trigger};
use crate::user::{is_active, login, logout};
use actix_cors::Cors;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::dev::Server;
use actix_web::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE};
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use dotenvy::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod agent;
mod errors;
pub mod helper;
pub mod models;
mod scheduler;
pub mod schema;
mod tmute;
mod user;

pub const APPLICATION_JSON: &str = "application/json";

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub const USER_ID_KEY: &'static str = "user_id";
pub const USER_HANDLE_KEY: &'static str = "user_handle";
pub const DID_KEY: &'static str = "did";
pub const ACTIVE_KEY: &'static str = "active";
pub const ACCESS_JWT_KEY: &'static str = "access_jwt";
pub const REFRESH_JWT_KEY: &'static str = "refresh_jwt";
pub const COOKIE_DATE_KEY: &'static str = "cookie_date";

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

fn create_cors(allowed_origin: &str) -> Cors {
    Cors::default()
        .allowed_origin(allowed_origin)
        .allowed_methods(vec!["GET", "POST", "OPTIONS"])
        .allowed_headers(vec![CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
        .supports_credentials()
        .max_age(3600)
}

fn create_cookie_middleware(
    secret_key: actix_web::cookie::Key,
) -> SessionMiddleware<CookieSessionStore> {
    let cookie_domain = env::var("COOKIE_DOMAIN").unwrap_or(".ripp.internal".to_string());
    let https_enabled = env::var("HTTPS_ENABLED").unwrap_or("1".to_string()).eq("1");
    const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;
    SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
        .cookie_content_security(CookieContentSecurity::Private)
        .cookie_name("bskytools".to_string())
        .session_lifecycle(
            PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
        )
        .cookie_http_only(true)
        .cookie_secure(https_enabled)
        .cookie_domain(Some(cookie_domain.to_string()))
        .cookie_same_site(actix_web::cookie::SameSite::Strict)
        .build()
}

fn init_db(database_url: &str, db_min_idle: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .min_idle(Some(db_min_idle.parse().unwrap()))
        .build(manager)
        .expect("Failed to create pool")
}

fn init_http_server(
    allowed_origin: String,
    pool: Pool<ConnectionManager<SqliteConnection>>,
    server_port: &str,
    worker_count: &str,
) -> Server {
    return HttpServer::new(move || {
        let secret_key = actix_web::cookie::Key::from(&[0; 64]);
        let cors = create_cors(allowed_origin.as_str());
        let cookie_middleware = create_cookie_middleware(secret_key);
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .wrap(cookie_middleware)
            .app_data(Data::new(pool.clone()))
            .service(login)
            .service(logout)
            .service(list)
            .service(create)
            .service(delete)
            .service(trigger)
            .service(is_active)
            .service(list_word)
            .service(create_word)
            .service(delete_word)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(format!("0.0.0.0:{}", server_port))
    .unwrap()
    .workers(worker_count.parse::<usize>().unwrap_or(2))
    .run();
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=debug");
    env_logger::init();

    // Get Environment Variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let db_min_idle = env::var("DB_MIN_IDLE").unwrap_or("1".to_string());
    let cron_schedule = env::var("CRON_SCHEDULE").unwrap_or("0 1 * * * * *".to_string());
    let allowed_origin =
        env::var("ALLOWED_ORIGIN").unwrap_or("http://frontend.ripp.internal".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("9090".to_string());
    let worker_count = env::var("WORKER_COUNT").unwrap_or("2".to_string());
    let cron_enabled = env::var("CRON_ENABLED").unwrap_or("0".to_string()) == "1";

    // Create DB Pool
    let db_pool = init_db(database_url.as_str(), db_min_idle.as_str());

    // Start Scheduler
    if cron_enabled {
        start_scheduler(cron_schedule.as_str()).await;
    }

    // Start Http Server
    let server = init_http_server(
        allowed_origin,
        db_pool,
        server_port.as_str(),
        worker_count.as_str(),
    );
    server.await
}
