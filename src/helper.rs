use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::{
    NewProfile, NewTimedMute, NewTimedMuteWord, Profile, TimedMute, TimedMuteWord,
};

pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_timed_mute(
    conn: &mut DBPooledConnection,
    actor: &str,
    muted_actor: &str,
    created_date: &i64,
    expiration_date: &i64,
    other_status: &i32,
) {
    use crate::schema::timed_mute;
    let new_timed_mute = NewTimedMute {
        actor,
        muted_actor,
        created_date,
        expiration_date,
        status: other_status,
    };

    let _ = diesel::insert_into(timed_mute::table)
        .values(&new_timed_mute)
        .execute(conn);
}

pub fn create_timed_mute_word(
    conn: &mut DBPooledConnection,
    actor: &str,
    muted_word: &str,
    created_date: &i64,
    expiration_date: &i64,
    status: &i32,
) {
    use crate::schema::timed_mute_word;
    let new_timed_mute = NewTimedMuteWord {
        actor,
        muted_word,
        created_date,
        expiration_date,
        status,
    };

    let _ = diesel::insert_into(timed_mute_word::table)
        .values(&new_timed_mute)
        .execute(conn);
}

pub fn update_timed_mute(
    conn: &mut DBPooledConnection,
    _actor: &str,
    _muted_actor: &str,
    expiration_time: &i64,
    status: &i32,
) -> bool {
    use crate::schema::timed_mute;

    let res = diesel::update(timed_mute::table)
        .filter(timed_mute::expiration_date.eq(expiration_time))
        .filter(timed_mute::actor.eq(_actor))
        .filter(timed_mute::muted_actor.eq(_muted_actor))
        .set(timed_mute::status.eq(status))
        .execute(conn)
        .expect("Error in updating timed mute");

    return res > 0;
}

pub fn update_timed_mute_word(
    conn: &mut DBPooledConnection,
    _actor: &str,
    _muted_word: &str,
    status: &i32,
) -> bool {
    use crate::schema::timed_mute_word;

    let res = diesel::update(timed_mute_word::table)
        .filter(timed_mute_word::actor.eq(_actor))
        .filter(timed_mute_word::muted_word.eq(_muted_word))
        .set(timed_mute_word::status.eq(status))
        .execute(conn)
        .expect("Error in updating timed mute word");

    return res > 0;
}

pub fn update_timed_mute_v1(
    conn: &mut SqliteConnection,
    _actor: &str,
    timed_mute_id: &i32,
    status: &i32,
) -> bool {
    use crate::schema::timed_mute;

    let res = diesel::update(timed_mute::table)
        .filter(timed_mute::rowid.eq(timed_mute_id))
        .filter(timed_mute::actor.eq(_actor))
        .set(timed_mute::status.eq(status))
        .execute(conn)
        .expect("Error in updating timed mute");

    return res > 0;
}

pub fn update_timed_mute_list_v1(
    conn: &mut SqliteConnection,
    _actor: &str,
    timed_mute_id_list: Vec<String>,
    status: &i32,
) -> bool {
    use crate::schema::timed_mute;

    let res = diesel::update(timed_mute::table)
        .filter(timed_mute::muted_actor.eq_any(timed_mute_id_list))
        .filter(timed_mute::actor.eq(_actor))
        .set(timed_mute::status.eq(status))
        .execute(conn)
        .expect("Error in updating timed mute");

    return res > 0;
}

pub fn update_timed_mute_word_list_v1(
    conn: &mut SqliteConnection,
    _actor: &str,
    timed_mute__word_list: Vec<String>,
    status: &i32,
) -> bool {
    use crate::schema::timed_mute_word;

    let res = diesel::update(timed_mute_word::table)
        .filter(timed_mute_word::muted_word.eq_any(timed_mute__word_list))
        .filter(timed_mute_word::actor.eq(_actor))
        .set(timed_mute_word::status.eq(status))
        .execute(conn)
        .expect("Error in updating timed mute word");

    return res > 0;
}

pub fn fetch_timed_mutes(conn: &mut DBPooledConnection, user_id: &str) -> Vec<TimedMute> {
    use crate::schema::timed_mute::actor;
    use crate::schema::timed_mute::dsl::timed_mute;
    use crate::schema::timed_mute::status;
    let results = timed_mute
        .filter(status.eq(0))
        .filter(actor.eq(user_id))
        .select(TimedMute::as_select())
        .load(conn)
        .expect("Error loading timed mutes");
    results
}

pub fn fetch_timed_mute_words(conn: &mut DBPooledConnection, user_id: &str) -> Vec<TimedMuteWord> {
    use crate::schema::timed_mute_word::actor;
    use crate::schema::timed_mute_word::dsl::timed_mute_word;
    use crate::schema::timed_mute_word::status;
    let results = timed_mute_word
        .filter(status.eq(0))
        .filter(actor.eq(user_id))
        .select(TimedMuteWord::as_select())
        .load(conn)
        .expect("Error loading timed mutes");
    results
}

pub fn fetch_timed_mutes_v1(conn: &mut SqliteConnection) -> Vec<TimedMute> {
    use crate::schema::timed_mute::dsl::timed_mute;
    use crate::schema::timed_mute::status;
    let results = timed_mute
        .filter(status.eq(0))
        .select(TimedMute::as_select())
        .load(conn)
        .expect("Error loading timed mutes");
    results
}

pub fn fetch_timed_mute_words_v1(conn: &mut SqliteConnection) -> Vec<TimedMuteWord> {
    use crate::schema::timed_mute_word::dsl::timed_mute_word;
    use crate::schema::timed_mute_word::status;
    let results = timed_mute_word
        .filter(status.eq(0))
        .select(TimedMuteWord::as_select())
        .load(conn)
        .expect("Error loading timed mutes");
    results
}

pub fn fetch_timed_mutes_for_user(conn: &mut DBPooledConnection, _actor: &str) -> Vec<TimedMute> {
    use crate::schema::timed_mute::actor;
    use crate::schema::timed_mute::dsl::timed_mute;
    use crate::schema::timed_mute::status;
    let results = timed_mute
        .filter(status.eq(0))
        .filter(actor.eq(_actor))
        .select(TimedMute::as_select())
        .load(conn)
        .expect("Error loading timed mutes");
    results
}

pub fn fetch_timed_mute_words_for_user(
    conn: &mut DBPooledConnection,
    _actor: &str,
) -> Vec<TimedMuteWord> {
    use crate::schema::timed_mute_word::actor;
    use crate::schema::timed_mute_word::dsl::timed_mute_word;
    use crate::schema::timed_mute_word::status;
    let results = timed_mute_word
        .filter(status.eq(0))
        .filter(actor.eq(_actor))
        .select(TimedMuteWord::as_select())
        .load(conn)
        .expect("Error loading timed mutes");
    results
}

pub fn fetch_profile(conn: &mut DBPooledConnection, _did: &str) -> Vec<Profile> {
    use crate::schema::profile::did;
    use crate::schema::profile::dsl::profile;
    let results = profile
        .filter(did.eq(_did))
        .select(Profile::as_select())
        .load::<Profile>(conn)
        .expect("DB Exception");
    results
}

pub fn fetch_profile_v1(conn: &mut SqliteConnection, _did: &str) -> Vec<Profile> {
    use crate::schema::profile::did;
    use crate::schema::profile::dsl::profile;
    let results = profile
        .filter(did.eq(_did))
        .select(Profile::as_select())
        .load::<Profile>(conn)
        .expect("DB Exception");
    results
}

pub fn create_profile(conn: &mut DBPooledConnection, did: &str, handle: &str, password: &str) {
    use crate::schema::profile;
    let new_profile = NewProfile {
        did,
        handle,
        password,
        status: &0,
    };

    let _ = diesel::insert_into(profile::table)
        .values(&new_profile)
        .execute(conn);
}

pub fn update_profile(conn: &mut DBPooledConnection, did: &str, _handle: &str, password: &str) {
    use crate::schema::profile;

    let _ = diesel::update(profile::table)
        .filter(profile::did.eq(did))
        .set(profile::password.eq(password))
        .execute(conn);
}

pub fn deactivate_profile(conn: &mut DBPooledConnection, did: &str) {
    use crate::schema::profile;

    let _ = diesel::update(profile::table)
        .filter(profile::did.eq(did))
        .set((profile::status.eq(&9), profile::password.eq("".to_string())))
        .execute(conn);
}
