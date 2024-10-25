#!/bin/sh
# wait-for-it.sh

set -e

host="$1"
port="$2"
shift 2
cmd="$@"

echo "Waiting for PostgreSQL at $host:$port..."
echo "Using credentials: $POSTGRES_USER / $POSTGRES_PASSWORD / $POSTGRES_DB"

until PGPASSWORD=$POSTGRES_PASSWORD psql -h "$host" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - executing command"
exec $cmd