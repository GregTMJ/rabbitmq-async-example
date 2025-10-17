-- Add down migration script here
ROLLBACK;

BEGIN;

DROP TABLE IF EXISTS application_requests;

DROP TABLE IF EXISTS application_responses;

DROP TABLE IF EXISTS fail_table;

DROP TABLE IF EXISTS service_requests;

DROP TABLE IF EXISTS service_responses;

DROP TABLE IF EXISTS services;

DROP TABLE IF EXISTS users;

COMMIT;