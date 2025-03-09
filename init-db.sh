#!/bin/bash
set -e

# Create the business_directory database
psql -v ON_ERROR_STOP=1 --username "$PGUSER" --dbname "$PGDB" <<-EOSQL
    CREATE DATABASE oplydb;
    CREATE DATABASE oplydbtest;
EOSQL

# Grant privileges
psql -v ON_ERROR_STOP=1 --username "$PGUSER" --dbname "$PGDB" -c "GRANT ALL PRIVILEGES ON DATABASE oplydb TO postgres;" 