#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: `psql` is not installed."
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    >&2 echo "Error: `sqlx` is not installed."
    >&2 echo "User:"
    >&2 echo "    cargo install sqlx-cli --no-default-features --features native-tls,postgres"
    >&2 echo "to install it."
    exit 1
fi


DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=indexer}"
DB_PORT="${POSTGRES_PORT:=5432}"

if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run -e POSTGRES_USER=${DB_USER} -e POSTGRES_PASSWORD=${DB_PASSWORD} -e POSTGRES_DB=${DB_NAME} -p "${DB_PORT}":5432 -d postgres postgres -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}";
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

>&2 echo "Postgress is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgress has been migrated, ready to go!"