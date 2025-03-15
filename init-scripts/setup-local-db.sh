#!/bin/bash
set -e

# Database credentials from .env file
source .env

# Create databases if they don't exist
PGPASSWORD=$PGPASSWORD psql -h localhost -U $PGUSER -d postgres <<-EOSQL
    CREATE DATABASE oplydb WITH OWNER = $PGUSER;
    CREATE DATABASE oplydbtest WITH OWNER = $PGUSER;
EOSQL

echo "Local databases created successfully!" 