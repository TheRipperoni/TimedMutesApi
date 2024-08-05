CREATE TABLE timed_mute_word (
    actor VARCHAR NOT NULL,
    muted_word VARCHAR NOT NULL,
    created_date BIGINT NOT NULL,
    expiration_date BIGINT NOT NULL,
    status INTEGER NOT NULL
);