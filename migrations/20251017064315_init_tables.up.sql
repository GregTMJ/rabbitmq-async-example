-- Add up migration script here
BEGIN;

CREATE TABLE IF NOT EXISTS services (
    id serial4 NOT NULL,
    "name" varchar(128) NULL,
    exchange varchar(128) NULL,
    queue varchar(128) NULL,
    routing_key varchar(128) NULL,
    cache_fields varchar(128) NULL,
    timeout int4 NULL,
    cache_expiration varchar NOT NULL,
    CONSTRAINT services_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS users (
    id serial4 NOT NULL,
    "name" varchar(128) NOT NULL,
    CONSTRAINT users_pkey PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS ix_users_name ON users USING btree (name);

-- Create the sequence first or use serial type
CREATE SEQUENCE IF NOT EXISTS applications_id_seq;

CREATE TABLE IF NOT EXISTS application_requests (
    id int4 DEFAULT nextval('applications_id_seq'::regclass) NOT NULL,
    service_id int4 NOT NULL,
    system_id int4 NOT NULL,
    application_id uuid DEFAULT gen_random_uuid() NOT NULL,
    timestamptz_saved timestamptz DEFAULT now() NULL,
    application_data jsonb NULL,
    serhub_request_id uuid DEFAULT gen_random_uuid() NOT NULL,
    CONSTRAINT applications_pkey PRIMARY KEY (id),
    CONSTRAINT applications_service_id_fkey FOREIGN KEY (service_id) REFERENCES services (id),
    CONSTRAINT applications_system_id_fkey FOREIGN KEY (system_id) REFERENCES users (id)
);

CREATE INDEX IF NOT EXISTS ix_application_requests_application_id ON application_requests USING btree (application_id);

CREATE UNIQUE INDEX IF NOT EXISTS ix_application_requests_serhub_request_id ON application_requests USING btree (serhub_request_id);

CREATE INDEX IF NOT EXISTS ix_application_requests_timestamptz_saved ON application_requests USING btree (timestamptz_saved);

-- Fixed: Changed IF EXISTS to IF NOT EXISTS
CREATE TABLE IF NOT EXISTS application_responses (
    id serial4 NOT NULL,
    application_id uuid NOT NULL,
    serhub_request_id uuid NOT NULL,
    service_id int4 NOT NULL,
    system_id int4 NOT NULL,
    is_cache bool NOT NULL,
    status varchar NOT NULL,
    response jsonb NULL,
    status_description jsonb NULL,
    target jsonb NULL,
    timestamptz_saved timestamptz DEFAULT now() NULL,
    CONSTRAINT application_responses_pkey PRIMARY KEY (id),
    CONSTRAINT application_responses_service_id_fkey FOREIGN KEY (service_id) REFERENCES services (id),
    CONSTRAINT application_responses_system_id_fkey FOREIGN KEY (system_id) REFERENCES users (id)
);

CREATE INDEX IF NOT EXISTS ix_application_responses_application_id ON application_responses USING btree (application_id);

CREATE UNIQUE INDEX IF NOT EXISTS ix_application_responses_serhub_request_id ON application_responses USING btree (serhub_request_id);

CREATE INDEX IF NOT EXISTS ix_application_responses_timestamptz_saved ON application_responses USING btree (timestamptz_saved);

CREATE TABLE IF NOT EXISTS fail_table (
    id serial4 NOT NULL,
    created_at timestamp NOT NULL,
    application_id uuid NOT NULL,
    service_id int4 NULL,
    system_id int4 NULL,
    error_type varchar NULL,
    error_message varchar NULL,
    error_traceback varchar NULL,
    "data" jsonb NULL,
    timestamptz_saved timestamptz DEFAULT now() NULL,
    serhub_request_id uuid NULL,
    CONSTRAINT fail_table_pkey PRIMARY KEY (id),
    CONSTRAINT fail_table_service_id_fkey FOREIGN KEY (service_id) REFERENCES services (id),
    CONSTRAINT fail_table_system_id_fkey FOREIGN KEY (system_id) REFERENCES users (id)
);

CREATE INDEX IF NOT EXISTS ix_fail_table_application_id ON fail_table USING btree (application_id);

CREATE INDEX IF NOT EXISTS ix_fail_table_created_at ON fail_table USING btree (created_at);

CREATE INDEX IF NOT EXISTS ix_fail_table_serhub_request_id ON fail_table USING btree (serhub_request_id);

CREATE INDEX IF NOT EXISTS ix_fail_table_timestamptz_saved ON fail_table USING btree (timestamptz_saved);

CREATE TABLE IF NOT EXISTS service_requests (
    id serial4 NOT NULL,
    application_id uuid NULL,
    service_id int4 NOT NULL,
    "data" text NOT NULL,
    timestamptz_saved timestamptz DEFAULT now() NOT NULL,
    serhub_request_id uuid NOT NULL,
    system_id int4 NULL,
    CONSTRAINT service_requests_pkey PRIMARY KEY (id),
    CONSTRAINT service_requests_foreign_system_id FOREIGN KEY (system_id) REFERENCES users (id),
    CONSTRAINT service_requests_service_id_fkey FOREIGN KEY (service_id) REFERENCES services (id)
);

CREATE INDEX IF NOT EXISTS ix_service_requests_application_id ON service_requests USING btree (application_id);

CREATE UNIQUE INDEX IF NOT EXISTS ix_service_requests_serhub_request_id ON service_requests USING btree (serhub_request_id);

CREATE INDEX IF NOT EXISTS ix_service_requests_timestamptz_saved ON service_requests USING btree (timestamptz_saved);

CREATE TABLE IF NOT EXISTS service_responses (
    id serial4 NOT NULL,
    application_id uuid NULL,
    service_id int4 NOT NULL,
    "data" text NULL,
    data_hash varchar NULL,
    is_cache bool NULL,
    timestamptz_saved timestamptz DEFAULT now() NOT NULL,
    serhub_request_id uuid NOT NULL,
    system_id int4 NULL,
    CONSTRAINT service_responses_pkey PRIMARY KEY (id),
    CONSTRAINT service_responses_foreign_system_id FOREIGN KEY (system_id) REFERENCES users (id),
    CONSTRAINT service_responses_service_id_fkey FOREIGN KEY (service_id) REFERENCES services (id)
);

CREATE INDEX IF NOT EXISTS ix_service_responses_application_id ON service_responses USING btree (application_id);

CREATE INDEX IF NOT EXISTS ix_service_responses_data_hash ON service_responses USING btree (data_hash);

CREATE UNIQUE INDEX IF NOT EXISTS ix_service_responses_serhub_request_id ON service_responses USING btree (serhub_request_id);

CREATE INDEX IF NOT EXISTS ix_service_responses_timestamptz_saved ON service_responses USING btree (timestamptz_saved);

COMMIT;