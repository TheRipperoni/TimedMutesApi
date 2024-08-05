use crate::agent::get_agent;
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
        (status=200, description="Successfully Logged In")
    ),
)]
#[post("/login")]
pub async fn login(
    pool: Data<DBPool>,
    req: Json<LoginRequest>,
    session: actix_session::Session,
) -> HttpResponse {
    let agent_res = get_agent(req.username.as_str(), req.password.as_str()).await;

    if agent_res.is_err() {
        return HttpResponse::BadRequest()
            .content_type(APPLICATION_JSON)
            .json({});
    }

    let agent = agent_res.expect("Unable to get agent");
    let bsky_session = agent.get_session().await.expect("Failed to get session");
    let mut conn = pool.get().expect("Connection pool error");
    let profiles = fetch_profile(&mut conn, bsky_session.did.as_str());
    if profiles.is_empty() {
        create_profile(
            &mut conn,
            bsky_session.did.as_str(),
            bsky_session.handle.as_str(),
            req.password.as_str(),
        );
    } else if profiles.get(0).unwrap().password.eq(&req.password) {
        update_profile(
            &mut conn,
            bsky_session.did.as_str(),
            bsky_session.handle.as_str(),
            req.password.as_str(),
        );
    }

    session.renew();
    session
        .insert(USER_ID_KEY, bsky_session.did.clone())
        .expect("User ID failed to insert");
    session
        .insert(USER_HANDLE_KEY, bsky_session.handle.clone())
        .expect("User handle failed to insert");
    session
        .insert(DID_KEY, bsky_session.did.to_string())
        .expect("DID failed to insert");
    session
        .insert(ACTIVE_KEY, bsky_session.active.unwrap())
        .expect("Active failed to insert");
    session
        .insert(ACCESS_JWT_KEY, bsky_session.access_jwt.clone())
        .expect("AccessJwt failed to insert");
    session
        .insert(REFRESH_JWT_KEY, bsky_session.refresh_jwt.clone())
        .expect("RefreshKey failed to insert");

    HttpResponse::Ok().content_type(APPLICATION_JSON).finish()
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
pub async fn deactivate(pool: Data<DBPool>, session: actix_session::Session) -> HttpResponse {
    let user_id: String;
    match session.get(USER_ID_KEY) {
        Ok(user_id_key) => match user_id_key {
            None => {
                return HttpResponse::Unauthorized().finish();
            }
            Some(id) => {
                user_id = id;
            }
        },
        Err(_e) => {
            return HttpResponse::InternalServerError().finish();
        }
    }
    let mut conn = pool.get().expect("Connection pool error");
    deactivate_profile(&mut conn, user_id.as_str());
    session.purge();
    HttpResponse::Ok().finish()
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
pub async fn is_active(session: actix_session::Session) -> HttpResponse {
    let result = session.get::<String>(USER_ID_KEY);
    if result.is_err() || result.unwrap().is_none() {
        return HttpResponse::Unauthorized().finish();
    }
    let body = IsActiveSuccessResponse {
        access_jwt: session.get::<String>(ACCESS_JWT_KEY).unwrap().unwrap(),
        refresh_jwt: session.get::<String>(REFRESH_JWT_KEY).unwrap().unwrap(),
        did: session.get::<String>(DID_KEY).unwrap().unwrap(),
        active: session.get::<bool>(ACTIVE_KEY).unwrap().unwrap(),
        handle: session.get::<String>(USER_HANDLE_KEY).unwrap().unwrap(),
    };
    HttpResponse::Ok().json(body)
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
