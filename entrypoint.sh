#!/bin/bash
set -e

# Create the database directory if it doesn't exist
DB_DIR=$(dirname $DATABASE_URL)
mkdir -p $DB_DIR 2>/dev/null || true

# Initialize the database with schema if it doesn't exist
if [ ! -f $DATABASE_URL ]; then
    echo "Creating database at $DATABASE_URL"
    sqlite3 $DATABASE_URL << 'SQL'
CREATE TABLE IF NOT EXISTS timed_mute (
    actor VARCHAR NOT NULL,
    muted_actor VARCHAR NOT NULL,
    created_date BIGINT NOT NULL,
    expiration_date BIGINT NOT NULL,
    status INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS profile (
    did VARCHAR NOT NULL,
    handle VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    status INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS timed_mute_word (
    actor VARCHAR NOT NULL,
    muted_word VARCHAR NOT NULL,
    created_date BIGINT NOT NULL,
    expiration_date BIGINT NOT NULL,
    status INTEGER NOT NULL
);
SQL
fi

# Run the app
exec ./TimedMutes
