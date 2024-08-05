CREATE TABLE timed_mute (
    actor VARCHAR NOT NULL,
    muted_actor VARCHAR NOT NULL,
    created_date BIGINT NOT NULL,
    expiration_date BIGINT NOT NULL,
    status INTEGER NOT NULL
);

CREATE TABLE profile (
    did VARCHAR NOT NULL,
    handle VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    status INTEGER NOT NULL
);