#!/bin/bash
set -e

# Check if PostgreSQL is running locally
pg_isready -h localhost -p 5432 -U postgres > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "PostgreSQL is not running locally. Please start PostgreSQL first."
    exit 1
fi

# Ensure databases exist
./setup-local-db.sh

# Set environment variable for local development
export USE_LOCAL_DB=true

# Run the backend in development mode
cd backend
cargo run