-- Add down migration script here
BEGIN;

TRUNCATE TABLE services CASCADE;

COMMIT;
