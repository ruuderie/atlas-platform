-- Default generic Anchor structural schema initialization
-- All data points MUST remain strictly structural. No personal configurations.

INSERT INTO "users" ("id", "username", "email", "password_hash", "status", "created_at", "updated_at") 
VALUES ('00000000-0000-0000-0000-000000000000', 'admin', 'admin@example.com', '', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;
