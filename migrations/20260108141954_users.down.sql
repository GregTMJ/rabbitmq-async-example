-- Add down migration script here
BEGIN;

TRUNCATE TABLE users CASCADE;

COMMIT;