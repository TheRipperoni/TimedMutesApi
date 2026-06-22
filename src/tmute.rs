use std::collections::HashMap;
use std::env;

use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use bsky_sdk::api::app::bsky::actor::get_profile::{Parameters, ParametersData};
use bsky_sdk::api::types::string::AtIdentifier;
use bsky_sdk::api::types::string::AtIdentifier::Handle;
use ipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use tower_sessions::Session;

use crate::agent::{
    add_mute_word_to_pref, get_agent, mute_actor, remove_mute_word_from_pref, unmute_actor,
};
use crate::error::AppError;
use crate::helper::{
    create_timed_mute, create_timed_mute_word, establish_connection, fetch_profile,
    fetch_profile_v1, fetch_timed_mute_words, fetch_timed_mute_words_v1, fetch_timed_mutes,
    fetch_timed_mutes_v1, update_timed_mute, update_timed_mute_list_v1, update_timed_mute_word,
    update_timed_mute_word_list_v1,
};
use crate::models::TimedMute;
use crate::{DBPool, APPLICATION_JSON, USER_ID_KEY};

async fn get_user_id(session: Session) -> Result<String, AppError> {
    session
        .get(USER_ID_KEY)
        .await
        .map_err(|_| AppError::InternalError)?
        .ok_or(AppError::Unauthorized)
}

#[utoipa::path(
    get,
    path = "/timed-mute-words",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="List of timed mute words", body = Vec<TimedMute>),
        (status=401, description="Unauthorized"),
    ),
)]
pub async fn list_word(
    session: Session,
    State(pool): State<DBPool>,
) -> Result<Response, AppError> {
    let user_id = get_user_id(session).await?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    let mute_list = fetch_timed_mute_words(&mut conn, user_id.as_str());
    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)],
        axum::Json(mute_list)
    ).into_response())
}

#[utoipa::path(
    get,
    path = "/timed-mutes",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="List of timed mutes", body = Vec<TimedMute>),
        (status=401, description="Unauthorized"),
    ),
)]
pub async fn list(
    session: Session,
    State(pool): State<DBPool>,
) -> Result<Response, AppError> {
    let user_id = get_user_id(session).await?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    let mute_list = fetch_timed_mutes(&mut conn, user_id.as_str());
    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)],
        axum::Json(mute_list)
    ).into_response())
}

#[utoipa::path(
    post,
    path = "/timed-mute",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="Timed mute successfully created"),
        (status=401, description="Unauthorized"),
    ),
)]
pub async fn create(
    session: Session,
    State(pool): State<DBPool>,
    Json(req): Json<CreateTimedMuteRequest>,
) -> Result<Response, AppError> {
    let user_id = get_user_id(session).await?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let create_time = chrono::offset::Utc::now().timestamp();
    let expire_time = create_time + req.expiration_length;

    let profile_list = fetch_profile_v1(&mut conn, user_id.as_str());
    let profile = profile_list.first().ok_or(AppError::NotFound)?;
    let agent = get_agent(profile.handle.as_str(), profile.password.as_str()).await?;

    let parsed_handle = req
        .muted_actor_handle
        .parse::<bsky_sdk::api::types::string::Handle>();
    if let Err(e) = parsed_handle {
        let response = BadHandle {
            error: e.to_string(),
        };
        return Ok((
            StatusCode::BAD_REQUEST,
            [(CONTENT_TYPE, APPLICATION_JSON)],
            axum::Json(response)
        ).into_response());
    }
    let other_handle: AtIdentifier = Handle(
        req.muted_actor_handle
            .parse()
            .map_err(|e| AppError::BskyError(format!("{:?}", e)))?,
    );
    let profile_data = agent
        .api
        .app
        .bsky
        .actor
        .get_profile(Parameters {
            data: ParametersData {
                actor: other_handle,
            },
            extra_data: Ipld::Null,
        })
        .await
        .map_err(|e| AppError::BskyError(e.to_string()))?;

    mute_actor(&agent, profile_data.did.as_str()).await?;

    create_timed_mute(
        &mut conn,
        user_id.as_str(),
        profile_data.did.as_str(),
        &create_time,
        &expire_time,
        &0,
    )?;
    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)]
    ).into_response())
}

pub async fn trigger() -> Response {
    resolve_timed_mutes().await;
    (
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)]
    ).into_response()
}

#[utoipa::path(
    post,
    path = "/deleteTimedMute",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="Successfully delete timed mute")
    ),
)]
pub async fn delete(
    session: Session,
    State(pool): State<DBPool>,
    Json(req): Json<DeleteTimedMuteRequest>,
) -> Result<Response, AppError> {
    let user_id = get_user_id(session).await?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let success = update_timed_mute(
        &mut conn,
        user_id.as_str(),
        req.muted_actor_did.as_str(),
        &req.expiration_date,
        &9,
    )?;
    if !success {
        return Err(AppError::Unauthorized);
    }
    let profile_list1 = fetch_profile(&mut conn, user_id.as_str());
    let profile1 = profile_list1.first().ok_or(AppError::NotFound)?;

    let agent_res = get_agent(profile1.handle.as_str(), profile1.password.as_str()).await?;

    unmute_actor(&agent_res, req.muted_actor_did.as_str()).await?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)]
    ).into_response())
}

#[utoipa::path(
    post,
    path = "/timed-mute-word",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="Timed mute word successfully created"),
        (status=401, description="Unauthorized"),
    ),
)]
pub async fn create_word(
    session: Session,
    State(pool): State<DBPool>,
    Json(req): Json<CreateTimedMuteWordRequest>,
) -> Result<Response, AppError> {
    let user_id = get_user_id(session).await?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let create_time = chrono::offset::Utc::now().timestamp();
    let expire_time = create_time + req.expiration_length;

    let profile_list = fetch_profile_v1(&mut conn, user_id.as_str());
    let profile = profile_list.first().ok_or(AppError::NotFound)?;
    let agent = get_agent(profile.handle.as_str(), profile.password.as_str()).await?;

    add_mute_word_to_pref(&agent, req.muted_word.clone()).await?;

    create_timed_mute_word(
        &mut conn,
        user_id.as_str(),
        req.muted_word.as_str(),
        &create_time,
        &expire_time,
        &0,
    )?;
    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)]
    ).into_response())
}

#[utoipa::path(
    post,
    path = "/deleteTimedMuteWord",
    params(
        ("bskytools" = String, Cookie,)
    ),
    responses(
        (status=200, description="Successfully delete timed mute")
    ),
)]
pub async fn delete_word(
    session: Session,
    State(pool): State<DBPool>,
    Json(req): Json<DeleteTimedMuteWordRequest>,
) -> Result<Response, AppError> {
    let user_id = get_user_id(session).await?;
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let success = update_timed_mute_word(&mut conn, user_id.as_str(), req.muted_word.as_str(), &9)?;
    if !success {
        return Err(AppError::Unauthorized);
    }
    let profile_list1 = fetch_profile(&mut conn, user_id.as_str());
    let profile1 = profile_list1.first().ok_or(AppError::NotFound)?;

    let agent = get_agent(profile1.handle.as_str(), profile1.password.as_str()).await?;

    remove_mute_word_from_pref(&agent, req.muted_word.clone()).await?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, APPLICATION_JSON)]
    ).into_response())
}

pub async fn resolve_timed_mutes() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let mut conn = establish_connection(database_url.as_str());
    let timed_mutes_res = fetch_timed_mutes_v1(&mut conn);
    let current_timestamp = chrono::offset::Utc::now().timestamp();
    let mut resolved_timed_mutes: HashMap<String, Vec<String>> = HashMap::new();

    for timed_mute in timed_mutes_res {
        if current_timestamp > timed_mute.expiration_date {
            if resolved_timed_mutes.contains_key(&timed_mute.actor) {
                let x = resolved_timed_mutes
                    .get_mut(&timed_mute.actor)
                    .expect("Error getting vector");
                x.push(timed_mute.muted_actor);
            } else {
                let actor = timed_mute.actor.clone();
                let x: Vec<String> = vec![timed_mute.muted_actor];
                resolved_timed_mutes.insert(actor, x);
            }
        }
    }

    for (key, value) in resolved_timed_mutes {
        let profile_list = fetch_profile_v1(&mut conn, key.as_str());
        let profile = match profile_list.first() {
            Some(p) => p,
            None => continue,
        };

        let agent_res = match get_agent(profile.handle.as_str(), profile.password.as_str()).await {
            Ok(a) => a,
            Err(_) => continue,
        };

        for actor_val in &value {
            let _ = unmute_actor(&agent_res, actor_val).await;
        }

        let _ = update_timed_mute_list_v1(&mut conn, key.as_str(), value, &1);
    }

    let timed_mute_words_res = fetch_timed_mute_words_v1(&mut conn);
    let mut resolved_timed_mute_words: HashMap<String, Vec<String>> = HashMap::new();
    for timed_mute_word in timed_mute_words_res {
        if current_timestamp > timed_mute_word.expiration_date {
            if resolved_timed_mute_words.contains_key(&timed_mute_word.actor) {
                let x = resolved_timed_mute_words
                    .get_mut(&timed_mute_word.actor)
                    .expect("Error getting vector");
                x.push(timed_mute_word.muted_word);
            } else {
                let actor = timed_mute_word.actor.clone();
                let x: Vec<String> = vec![timed_mute_word.muted_word];
                resolved_timed_mute_words.insert(actor, x);
            }
        }
    }

    for (key, value) in resolved_timed_mute_words {
        let profile_list = fetch_profile_v1(&mut conn, key.as_str());
        let profile = match profile_list.first() {
            Some(p) => p,
            None => continue,
        };

        let agent_res = match get_agent(profile.handle.as_str(), profile.password.as_str()).await {
            Ok(a) => a,
            Err(_) => continue,
        };

        for muted_word in &value {
            let _ = remove_mute_word_from_pref(&agent_res, muted_word.to_string()).await;
        }

        let _ = update_timed_mute_word_list_v1(&mut conn, key.as_str(), value, &1);
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTimedMuteRequest {
    pub muted_actor_handle: String,
    pub expiration_length: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateTimedMuteWordRequest {
    pub muted_word: String,
    pub expiration_length: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteTimedMuteRequest {
    pub muted_actor_did: String,
    pub expiration_date: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteTimedMuteWordRequest {
    pub muted_word: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BadHandle {
    pub error: String,
}
