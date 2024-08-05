use std::collections::HashMap;
use std::env;

use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpResponse};
use bsky_sdk::api::app::bsky::actor::get_profile::{Parameters, ParametersData};
use bsky_sdk::api::types::string::AtIdentifier;
use bsky_sdk::api::types::string::AtIdentifier::Handle;
use ipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::agent::{
    add_mute_word_to_pref, get_agent, mute_actor, remove_mute_word_from_pref, unmute_actor,
    UnmuteActorResult,
};
use crate::helper::{
    create_timed_mute, create_timed_mute_word, establish_connection, fetch_profile,
    fetch_profile_v1, fetch_timed_mute_words, fetch_timed_mute_words_v1, fetch_timed_mutes,
    fetch_timed_mutes_v1, update_timed_mute, update_timed_mute_list_v1, update_timed_mute_word,
    update_timed_mute_word_list_v1,
};
use crate::{DBPool, APPLICATION_JSON, USER_ID_KEY};

fn get_user_id(session: actix_session::Session) -> Result<String, HttpResponse> {
    return match session.get(USER_ID_KEY) {
        Ok(user_id_key) => match user_id_key {
            None => Err(HttpResponse::Unauthorized().finish()),
            Some(id) => Ok(id),
        },
        Err(_e) => Err(HttpResponse::InternalServerError().finish()),
    };
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
#[get("/timed-mute-words")]
pub async fn list_word(session: actix_session::Session, pool: Data<DBPool>) -> HttpResponse {
    let user_id: String;
    match get_user_id(session) {
        Ok(val) => user_id = val,
        Err(val) => return val,
    }
    let mut conn = pool.get().expect("Connection pool error");
    let mute_list = fetch_timed_mute_words(&mut conn, user_id.as_str());
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(mute_list)
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
#[get("/timed-mutes")]
pub async fn list(session: actix_session::Session, pool: Data<DBPool>) -> HttpResponse {
    let user_id: String;
    match get_user_id(session) {
        Ok(val) => user_id = val,
        Err(val) => return val,
    }
    let mut conn = pool.get().expect("Connection pool error");
    let mute_list = fetch_timed_mutes(&mut conn, user_id.as_str());
    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(mute_list)
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
#[post("/timed-mute")]
pub async fn create(
    session: actix_session::Session,
    pool: Data<DBPool>,
    req: Json<CreateTimedMuteRequest>,
) -> HttpResponse {
    let user_id: String;
    match get_user_id(session) {
        Ok(val) => user_id = val,
        Err(val) => return val,
    }
    let mut conn = pool.get().expect("Connection pool error");

    let create_time = chrono::offset::Utc::now().timestamp();
    let expire_time = create_time + req.expiration_length;

    let profile_list = fetch_profile_v1(&mut conn, user_id.as_str());
    let profile = profile_list.get(0).unwrap();
    let agent = get_agent(profile.handle.as_str(), profile.password.as_str())
        .await
        .unwrap();

    let parsed_handle = req
        .muted_actor_handle
        .parse::<bsky_sdk::api::types::string::Handle>();
    if parsed_handle.is_err() {
        let response = BadHandle {
            error: parsed_handle.err().unwrap().to_string(),
        };
        return HttpResponse::BadRequest().json(response);
    }
    let other_handle: AtIdentifier =
        Handle(req.muted_actor_handle.parse().expect("something wrong"));
    let profile = agent
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
        .unwrap();

    mute_actor(&agent, profile.did.as_str())
        .await
        .expect("Failed to mute actor");

    create_timed_mute(
        &mut conn,
        user_id.as_str(),
        profile.did.as_str(),
        &create_time,
        &expire_time,
        &0,
    );
    HttpResponse::Ok().content_type(APPLICATION_JSON).finish()
}

#[post("/trigger")]
pub async fn trigger() -> HttpResponse {
    resolve_timed_mutes().await;
    HttpResponse::Ok().content_type(APPLICATION_JSON).finish()
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
#[post("/deleteTimedMute")]
pub async fn delete(
    session: actix_session::Session,
    pool: Data<DBPool>,
    req: Json<DeleteTimedMuteRequest>,
) -> HttpResponse {
    let user_id: String;
    match get_user_id(session) {
        Ok(val) => user_id = val,
        Err(val) => return val,
    }
    let mut conn = pool.get().expect("Connection pool error");

    let success = update_timed_mute(
        &mut conn,
        user_id.as_str(),
        req.muted_actor_did.as_str(),
        &req.expiration_date,
        &9,
    );
    if !success {
        return HttpResponse::Unauthorized().finish();
    }
    let profile_list1 = fetch_profile(&mut conn, user_id.as_str());
    let profile1 = profile_list1.get(0).unwrap();

    let agent_res = get_agent(profile1.handle.as_str(), profile1.password.as_str())
        .await
        .unwrap();

    let res: UnmuteActorResult = unmute_actor(&agent_res, req.muted_actor_did.as_str()).await;
    if res.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().content_type(APPLICATION_JSON).finish()
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
#[post("/timed-mute-word")]
pub async fn create_word(
    session: actix_session::Session,
    pool: Data<DBPool>,
    req: Json<CreateTimedMuteWordRequest>,
) -> HttpResponse {
    let user_id: String;
    match get_user_id(session) {
        Ok(val) => user_id = val,
        Err(val) => return val,
    }
    let mut conn = pool.get().expect("Connection pool error");

    let create_time = chrono::offset::Utc::now().timestamp();
    let expire_time = create_time + req.expiration_length;

    let profile_list = fetch_profile_v1(&mut conn, user_id.as_str());
    let profile = profile_list.get(0).unwrap();
    let agent = get_agent(profile.handle.as_str(), profile.password.as_str())
        .await
        .unwrap();

    add_mute_word_to_pref(&agent, req.muted_word.clone()).await;

    create_timed_mute_word(
        &mut conn,
        user_id.as_str(),
        req.muted_word.as_str(),
        &create_time,
        &expire_time,
        &0,
    );

    HttpResponse::Ok().content_type(APPLICATION_JSON).finish()
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
#[post("/deleteTimedMuteWord")]
pub async fn delete_word(
    session: actix_session::Session,
    pool: Data<DBPool>,
    req: Json<DeleteTimedMuteWordRequest>,
) -> HttpResponse {
    let user_id: String;
    match get_user_id(session) {
        Ok(val) => user_id = val,
        Err(val) => return val,
    }
    let mut conn = pool.get().expect("Connection pool error");

    let success = update_timed_mute_word(&mut conn, user_id.as_str(), req.muted_word.as_str(), &9);
    if !success {
        return HttpResponse::Unauthorized().finish();
    }
    let profile_list1 = fetch_profile(&mut conn, user_id.as_str());
    let profile1 = profile_list1.get(0).unwrap();

    let agent = get_agent(profile1.handle.as_str(), profile1.password.as_str())
        .await
        .unwrap();

    remove_mute_word_from_pref(&agent, req.muted_word.clone()).await;

    HttpResponse::Ok().content_type(APPLICATION_JSON).finish()
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
                let mut x: Vec<String> = Vec::new();
                x.push(timed_mute.muted_actor);
                resolved_timed_mutes.insert(actor, x);
            }
        }
    }

    for (key, value) in resolved_timed_mutes {
        let profile_list = fetch_profile_v1(&mut conn, key.as_str());
        let profile = profile_list.get(0).unwrap();

        let agent_res = get_agent(profile.handle.as_str(), profile.password.as_str())
            .await
            .unwrap();

        for actor_val in &value {
            unmute_actor(&agent_res, actor_val)
                .await
                .expect("Failed to unmute actor");
        }

        update_timed_mute_list_v1(&mut conn, key.as_str(), value, &1);
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
                let mut x: Vec<String> = Vec::new();
                x.push(timed_mute_word.muted_word);
                resolved_timed_mute_words.insert(actor, x);
            }
        }
    }

    for (key, value) in resolved_timed_mute_words {
        let profile_list = fetch_profile_v1(&mut conn, key.as_str());
        let profile = profile_list.get(0).unwrap();

        let agent_res = get_agent(profile.handle.as_str(), profile.password.as_str())
            .await
            .unwrap();

        for muted_word in &value {
            remove_mute_word_from_pref(&agent_res, muted_word.to_string()).await;
        }

        update_timed_mute_word_list_v1(&mut conn, key.as_str(), value, &1);
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
