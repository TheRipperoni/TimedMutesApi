use crate::agent::get_agent;
use crate::error::AppError;
use crate::helper::{create_profile, deactivate_profile, fetch_profile, update_profile};
use crate::{
    DBPool, ACCESS_JWT_KEY, ACTIVE_KEY, APPLICATION_JSON, DID_KEY, REFRESH_JWT_KEY,
    USER_HANDLE_KEY, USER_ID_KEY,
};
use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/login",
    responses(
        (status=200, description="Successfully Logged In"),
        (status=400, description="Bad Request"),
        (status=500, description="Internal Server Error")
    ),
)]
#[post("/login")]
pub async fn login(
    pool: Data<DBPool>,
    req: Json<LoginRequest>,
    session: actix_session::Session,
) -> Result<HttpResponse, AppError> {
    let agent = get_agent(req.username.as_str(), req.password.as_str()).await?;

    let bsky_session = agent
        .get_session()
        .await
        .ok_or_else(|| AppError::BskyError("Failed to get session".to_string()))?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    let profiles = fetch_profile(&mut conn, bsky_session.did.as_str());
    if profiles.is_empty() {
        create_profile(
            &mut conn,
            bsky_session.did.as_str(),
            bsky_session.handle.as_str(),
            req.password.as_str(),
        )?;
    } else if profiles.first().unwrap().password.eq(&req.password) {
        update_profile(
            &mut conn,
            bsky_session.did.as_str(),
            bsky_session.handle.as_str(),
            req.password.as_str(),
        )?;
    }

    session.renew();
    session
        .insert(USER_ID_KEY, bsky_session.did.clone())
        .map_err(|_| AppError::InternalError)?;
    session
        .insert(USER_HANDLE_KEY, bsky_session.handle.clone())
        .map_err(|_| AppError::InternalError)?;
    session
        .insert(DID_KEY, bsky_session.did.to_string())
        .map_err(|_| AppError::InternalError)?;
    session
        .insert(ACTIVE_KEY, bsky_session.active.unwrap_or(false))
        .map_err(|_| AppError::InternalError)?;
    session
        .insert(ACCESS_JWT_KEY, bsky_session.access_jwt.clone())
        .map_err(|_| AppError::InternalError)?;
    session
        .insert(REFRESH_JWT_KEY, bsky_session.refresh_jwt.clone())
        .map_err(|_| AppError::InternalError)?;

    Ok(HttpResponse::Ok().content_type(APPLICATION_JSON).finish())
}

#[utoipa::path(
    post,
    path = "/logout",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="Successfully Logged Out")
    ),
)]
#[post("/logout")]
pub async fn logout(session: actix_session::Session) -> HttpResponse {
    session.purge();
    HttpResponse::Ok().finish()
}

#[post("/deactivate")]
pub async fn deactivate(
    pool: Data<DBPool>,
    session: actix_session::Session,
) -> Result<HttpResponse, AppError> {
    let user_id: String = session
        .get(USER_ID_KEY)
        .map_err(|_| AppError::InternalError)?
        .ok_or(AppError::Unauthorized)?;

    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    deactivate_profile(&mut conn, user_id.as_str())?;
    session.purge();
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    get,
    path = "/active",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="Active Session", body = IsActiveSuccessResponse),
        (status=401, description="Unauthorized/Not Logged In"),
    ),
)]
#[get("/active")]
pub async fn is_active(session: actix_session::Session) -> Result<HttpResponse, AppError> {
    let _ = session
        .get::<String>(USER_ID_KEY)
        .map_err(|_| AppError::InternalError)?
        .ok_or(AppError::Unauthorized)?;

    let body = IsActiveSuccessResponse {
        access_jwt: session
            .get::<String>(ACCESS_JWT_KEY)
            .map_err(|_| AppError::InternalError)?
            .ok_or(AppError::Unauthorized)?,
        refresh_jwt: session
            .get::<String>(REFRESH_JWT_KEY)
            .map_err(|_| AppError::InternalError)?
            .ok_or(AppError::Unauthorized)?,
        did: session
            .get::<String>(DID_KEY)
            .map_err(|_| AppError::InternalError)?
            .ok_or(AppError::Unauthorized)?,
        active: session
            .get::<bool>(ACTIVE_KEY)
            .map_err(|_| AppError::InternalError)?
            .ok_or(AppError::Unauthorized)?,
        handle: session
            .get::<String>(USER_HANDLE_KEY)
            .map_err(|_| AppError::InternalError)?
            .ok_or(AppError::Unauthorized)?,
    };
    Ok(HttpResponse::Ok().json(body))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct IsActiveSuccessResponse {
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub did: String,
    pub active: bool,
    pub handle: String,
}
