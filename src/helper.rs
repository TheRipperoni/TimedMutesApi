use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::error::AppError;
use crate::models::{
    NewProfile, NewTimedMute, NewTimedMuteWord, Profile, TimedMute, TimedMuteWord,
};

pub type Result<T> = std::result::Result<T, AppError>;
pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::r2d2::{ConnectionManager, Pool};

    fn setup_test_pool() -> DBPool {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        let mut conn = pool.get().unwrap();

        diesel::sql_query(
            "CREATE TABLE timed_mute (
            actor VARCHAR NOT NULL,
            muted_actor VARCHAR NOT NULL,
            created_date BIGINT NOT NULL,
            expiration_date BIGINT NOT NULL,
            status INTEGER NOT NULL
        )",
        )
        .execute(&mut conn)
        .unwrap();

        diesel::sql_query(
            "CREATE TABLE profile (
            did VARCHAR NOT NULL,
            handle VARCHAR NOT NULL,
            password VARCHAR NOT NULL,
            status INTEGER NOT NULL
        )",
        )
        .execute(&mut conn)
        .unwrap();

        diesel::sql_query(
            "CREATE TABLE timed_mute_word (
            actor VARCHAR NOT NULL,
            muted_word VARCHAR NOT NULL,
            created_date BIGINT NOT NULL,
            expiration_date BIGINT NOT NULL,
            status INTEGER NOT NULL
        )",
        )
        .execute(&mut conn)
        .unwrap();

        pool
    }

    #[test]
    fn test_profile_operations() {
        let pool = setup_test_pool();
        let mut conn = pool.get().unwrap();

        let did = "did:plc:123";
        let handle = "test.bsky.social";
        let password = "password123";

        // Test create_profile
        let _ = create_profile(&mut conn, did, handle, password).unwrap();

        // Test fetch_profile
        let profiles = fetch_profile(&mut conn, did);
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].did, did);
        assert_eq!(profiles[0].handle, handle);
        assert_eq!(profiles[0].password, password);
        assert_eq!(profiles[0].status, 0);

        // Test update_profile
        let new_password = "newpassword";
        let _ = update_profile(&mut conn, did, handle, new_password).unwrap();
        let profiles = fetch_profile(&mut conn, did);
        assert_eq!(profiles[0].password, new_password);

        // Test deactivate_profile
        let _ = deactivate_profile(&mut conn, did).unwrap();
        let profiles = fetch_profile(&mut conn, did);
        assert_eq!(profiles[0].status, 9);
        assert_eq!(profiles[0].password, "");
    }

    #[test]
    fn test_timed_mute_operations() {
        let pool = setup_test_pool();
        let mut conn = pool.get().unwrap();

        let actor = "did:plc:actor";
        let muted_actor = "did:plc:muted";
        let created_date = 1000;
        let expiration_date = 2000;
        let status = 0;

        // Test create_timed_mute
        let _ = create_timed_mute(
            &mut conn,
            actor,
            muted_actor,
            &created_date,
            &expiration_date,
            &status,
        )
        .unwrap();

        // Test fetch_timed_mutes
        let mutes = fetch_timed_mutes(&mut conn, actor);
        assert_eq!(mutes.len(), 1);
        assert_eq!(mutes[0].muted_actor, muted_actor);

        // Test update_timed_mute
        let new_status = 1;
        let updated =
            update_timed_mute(&mut conn, actor, muted_actor, &expiration_date, &new_status)
                .unwrap();
        assert!(updated);

        let mutes = fetch_timed_mutes_for_user(&mut conn, actor);
        // fetch_timed_mutes filters by status = 0, so it should be empty now
        assert_eq!(mutes.len(), 0);
    }

    #[test]
    fn test_timed_mute_word_operations() {
        let pool = setup_test_pool();
        let mut conn = pool.get().unwrap();

        let actor = "did:plc:actor";
        let muted_word = "badword";
        let created_date = 1000;
        let expiration_date = 2000;
        let status = 0;

        // Test create_timed_mute_word
        let _ = create_timed_mute_word(
            &mut conn,
            actor,
            muted_word,
            &created_date,
            &expiration_date,
            &status,
        )
        .unwrap();

        // Test fetch_timed_mute_words
        let words = fetch_timed_mute_words(&mut conn, actor);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].muted_word, muted_word);

        // Test update_timed_mute_word
        let new_status = 1;
        let updated = update_timed_mute_word(&mut conn, actor, muted_word, &new_status).unwrap();
        assert!(updated);

        let words = fetch_timed_mute_words_for_user(&mut conn, actor);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_v1_operations() {
        let pool = setup_test_pool();
        let mut conn = pool.get().unwrap();

        let actor = "did:plc:actor";
        let _ = create_timed_mute(&mut conn, actor, "muted1", &1000, &2000, &0).unwrap();
        let _ = create_timed_mute_word(&mut conn, actor, "word1", &1000, &2000, &0).unwrap();

        let _ = create_profile(&mut conn, "did1", "handle1", "pass1").unwrap();

        let p = fetch_profile_v1(&mut conn, "did1");
        assert_eq!(p.len(), 1);

        let mutes = fetch_timed_mutes_v1(&mut conn);
        assert_eq!(mutes.len(), 1);

        let words = fetch_timed_mute_words_v1(&mut conn);
        assert_eq!(words.len(), 1);

        let _ =
            update_timed_mute_list_v1(&mut conn, actor, vec!["muted1".to_string()], &1).unwrap();
        let mutes = fetch_timed_mutes_v1(&mut conn);
        assert_eq!(mutes.len(), 0);

        let _ = update_timed_mute_word_list_v1(&mut conn, actor, vec!["word1".to_string()], &1)
            .unwrap();
        let words = fetch_timed_mute_words_v1(&mut conn);
        assert_eq!(words.len(), 0);
    }
}

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_timed_mute(
    conn: &mut DBPooledConnection,
    actor: &str,
    muted_actor: &str,
    created_date: &i64,
    expiration_date: &i64,
    other_status: &i32,
) -> Result<usize> {
    use crate::schema::timed_mute;
    let new_timed_mute = NewTimedMute {
        actor,
        muted_actor,
        created_date,
        expiration_date,
        status: other_status,
    };

    diesel::insert_into(timed_mute::table)
        .values(&new_timed_mute)
        .execute(conn)
        .map_err(AppError::from)
}

pub fn create_timed_mute_word(
    conn: &mut DBPooledConnection,
    actor: &str,
    muted_word: &str,
    created_date: &i64,
    expiration_date: &i64,
    status: &i32,
) -> Result<usize> {
    use crate::schema::timed_mute_word;
    let new_timed_mute = NewTimedMuteWord {
        actor,
        muted_word,
        created_date,
        expiration_date,
        status,
    };

    diesel::insert_into(timed_mute_word::table)
        .values(&new_timed_mute)
        .execute(conn)
        .map_err(AppError::from)
}

pub fn update_timed_mute(
    conn: &mut DBPooledConnection,
    _actor: &str,
    _muted_actor: &str,
    expiration_time: &i64,
    status: &i32,
) -> Result<bool> {
    use crate::schema::timed_mute;

    let res = diesel::update(timed_mute::table)
        .filter(timed_mute::expiration_date.eq(expiration_time))
        .filter(timed_mute::actor.eq(_actor))
        .filter(timed_mute::muted_actor.eq(_muted_actor))
        .set(timed_mute::status.eq(status))
        .execute(conn)?;

    Ok(res > 0)
}

pub fn update_timed_mute_word(
    conn: &mut DBPooledConnection,
    _actor: &str,
    _muted_word: &str,
    status: &i32,
) -> Result<bool> {
    use crate::schema::timed_mute_word;

    let res = diesel::update(timed_mute_word::table)
        .filter(timed_mute_word::actor.eq(_actor))
        .filter(timed_mute_word::muted_word.eq(_muted_word))
        .set(timed_mute_word::status.eq(status))
        .execute(conn)?;

    Ok(res > 0)
}

pub fn update_timed_mute_v1(
    conn: &mut SqliteConnection,
    _actor: &str,
    timed_mute_id: &i32,
    status: &i32,
) -> Result<bool> {
    use crate::schema::timed_mute;

    let res = diesel::update(timed_mute::table)
        .filter(timed_mute::rowid.eq(timed_mute_id))
        .filter(timed_mute::actor.eq(_actor))
        .set(timed_mute::status.eq(status))
        .execute(conn)?;

    Ok(res > 0)
}

pub fn update_timed_mute_list_v1(
    conn: &mut SqliteConnection,
    _actor: &str,
    timed_mute_id_list: Vec<String>,
    status: &i32,
) -> Result<bool> {
    use crate::schema::timed_mute;

    let res = diesel::update(timed_mute::table)
        .filter(timed_mute::muted_actor.eq_any(timed_mute_id_list))
        .filter(timed_mute::actor.eq(_actor))
        .set(timed_mute::status.eq(status))
        .execute(conn)?;

    Ok(res > 0)
}

pub fn update_timed_mute_word_list_v1(
    conn: &mut SqliteConnection,
    _actor: &str,
    timed_mute_word_list: Vec<String>,
    status: &i32,
) -> Result<bool> {
    use crate::schema::timed_mute_word;

    let res = diesel::update(timed_mute_word::table)
        .filter(timed_mute_word::muted_word.eq_any(timed_mute_word_list))
        .filter(timed_mute_word::actor.eq(_actor))
        .set(timed_mute_word::status.eq(status))
        .execute(conn)?;

    Ok(res > 0)
}

pub fn fetch_timed_mutes(conn: &mut DBPooledConnection, user_id: &str) -> Vec<TimedMute> {
    use crate::schema::timed_mute::actor;
    use crate::schema::timed_mute::dsl::timed_mute;
    use crate::schema::timed_mute::status;
    timed_mute
        .filter(status.eq(0))
        .filter(actor.eq(user_id))
        .select(TimedMute::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn fetch_timed_mute_words(conn: &mut DBPooledConnection, user_id: &str) -> Vec<TimedMuteWord> {
    use crate::schema::timed_mute_word::actor;
    use crate::schema::timed_mute_word::dsl::timed_mute_word;
    use crate::schema::timed_mute_word::status;
    timed_mute_word
        .filter(status.eq(0))
        .filter(actor.eq(user_id))
        .select(TimedMuteWord::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn fetch_timed_mutes_v1(conn: &mut SqliteConnection) -> Vec<TimedMute> {
    use crate::schema::timed_mute::dsl::timed_mute;
    use crate::schema::timed_mute::status;
    timed_mute
        .filter(status.eq(0))
        .select(TimedMute::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn fetch_timed_mute_words_v1(conn: &mut SqliteConnection) -> Vec<TimedMuteWord> {
    use crate::schema::timed_mute_word::dsl::timed_mute_word;
    use crate::schema::timed_mute_word::status;
    timed_mute_word
        .filter(status.eq(0))
        .select(TimedMuteWord::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn fetch_timed_mutes_for_user(conn: &mut DBPooledConnection, _actor: &str) -> Vec<TimedMute> {
    use crate::schema::timed_mute::actor;
    use crate::schema::timed_mute::dsl::timed_mute;
    use crate::schema::timed_mute::status;
    timed_mute
        .filter(status.eq(0))
        .filter(actor.eq(_actor))
        .select(TimedMute::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn fetch_timed_mute_words_for_user(
    conn: &mut DBPooledConnection,
    _actor: &str,
) -> Vec<TimedMuteWord> {
    use crate::schema::timed_mute_word::actor;
    use crate::schema::timed_mute_word::dsl::timed_mute_word;
    use crate::schema::timed_mute_word::status;
    timed_mute_word
        .filter(status.eq(0))
        .filter(actor.eq(_actor))
        .select(TimedMuteWord::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn fetch_profile(conn: &mut DBPooledConnection, _did: &str) -> Vec<Profile> {
    use crate::schema::profile::did;
    use crate::schema::profile::dsl::profile;
    profile
        .filter(did.eq(_did))
        .select(Profile::as_select())
        .load::<Profile>(conn)
        .unwrap_or_default()
}

pub fn fetch_profile_v1(conn: &mut SqliteConnection, _did: &str) -> Vec<Profile> {
    use crate::schema::profile::did;
    use crate::schema::profile::dsl::profile;
    profile
        .filter(did.eq(_did))
        .select(Profile::as_select())
        .load::<Profile>(conn)
        .unwrap_or_default()
}

pub fn create_profile(
    conn: &mut DBPooledConnection,
    did: &str,
    handle: &str,
    password: &str,
) -> Result<usize> {
    use crate::schema::profile;
    let new_profile = NewProfile {
        did,
        handle,
        password,
        status: &0,
    };

    diesel::insert_into(profile::table)
        .values(&new_profile)
        .execute(conn)
        .map_err(AppError::from)
}

pub fn update_profile(
    conn: &mut DBPooledConnection,
    did: &str,
    _handle: &str,
    password: &str,
) -> Result<usize> {
    use crate::schema::profile;

    diesel::update(profile::table)
        .filter(profile::did.eq(did))
        .set(profile::password.eq(password))
        .execute(conn)
        .map_err(AppError::from)
}

pub fn deactivate_profile(conn: &mut DBPooledConnection, did: &str) -> Result<usize> {
    use crate::schema::profile;

    diesel::update(profile::table)
        .filter(profile::did.eq(did))
        .set((profile::status.eq(&9), profile::password.eq("".to_string())))
        .execute(conn)
        .map_err(AppError::from)
}
