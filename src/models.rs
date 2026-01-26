use crate::schema::profile;
use crate::schema::timed_mute;
use crate::schema::timed_mute_word;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Queryable, Selectable, Debug, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::timed_mute)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TimedMute {
    pub actor: String,
    pub muted_actor: String,
    pub created_date: i64,
    pub expiration_date: i64,
    pub status: i32,
}

impl TimedMute {
    pub fn new(
        actor: String,
        muted_actor: String,
        created_date: i64,
        expiration_date: i64,
        status: i32,
    ) -> Self {
        Self {
            actor,
            muted_actor,
            created_date,
            expiration_date,
            status,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = timed_mute)]
pub struct NewTimedMute<'a> {
    pub actor: &'a str,
    pub muted_actor: &'a str,
    pub created_date: &'a i64,
    pub expiration_date: &'a i64,
    pub status: &'a i32,
}

#[derive(Queryable, Selectable, Debug, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::profile)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Profile {
    pub did: String,
    pub handle: String,
    pub password: String,
    pub status: i32,
}

impl Profile {
    pub fn new(did: String, handle: String, password: String, status: i32) -> Self {
        Self {
            did,
            handle,
            password,
            status,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = profile)]
pub struct NewProfile<'a> {
    pub did: &'a str,
    pub handle: &'a str,
    pub password: &'a str,
    pub status: &'a i32,
}

#[derive(Queryable, Selectable, Debug, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::timed_mute_word)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TimedMuteWord {
    pub actor: String,
    pub muted_word: String,
    pub created_date: i64,
    pub expiration_date: i64,
    pub status: i32,
}

impl TimedMuteWord {
    pub fn new(
        actor: String,
        muted_word: String,
        created_date: i64,
        expiration_date: i64,
        status: i32,
    ) -> Self {
        Self {
            actor,
            muted_word,
            created_date,
            expiration_date,
            status,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = timed_mute_word)]
pub struct NewTimedMuteWord<'a> {
    pub actor: &'a str,
    pub muted_word: &'a str,
    pub created_date: &'a i64,
    pub expiration_date: &'a i64,
    pub status: &'a i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timed_mute_new() {
        let tm = TimedMute::new("actor1".to_string(), "muted1".to_string(), 1000, 2000, 0);
        assert_eq!(tm.actor, "actor1");
        assert_eq!(tm.muted_actor, "muted1");
        assert_eq!(tm.created_date, 1000);
        assert_eq!(tm.expiration_date, 2000);
        assert_eq!(tm.status, 0);
    }

    #[test]
    fn test_profile_new() {
        let p = Profile::new(
            "did1".to_string(),
            "handle1".to_string(),
            "pass1".to_string(),
            0,
        );
        assert_eq!(p.did, "did1");
        assert_eq!(p.handle, "handle1");
        assert_eq!(p.password, "pass1");
        assert_eq!(p.status, 0);
    }

    #[test]
    fn test_timed_mute_word_new() {
        let tmw = TimedMuteWord::new("actor1".to_string(), "word1".to_string(), 1000, 2000, 0);
        assert_eq!(tmw.actor, "actor1");
        assert_eq!(tmw.muted_word, "word1");
        assert_eq!(tmw.created_date, 1000);
        assert_eq!(tmw.expiration_date, 2000);
        assert_eq!(tmw.status, 0);
    }
}
