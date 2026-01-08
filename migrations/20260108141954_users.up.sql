-- Add up migration script here
BEGIN;

-- Insert data into the `users` table
INSERT INTO
    users (id, name)
VALUES
    (1, 'Bistrodengi'),
    (2, 'Scortech'),
    (3, 'Turbozaim'),
    (4, 'FO'),
    (5, 'Eqvantalab'),
    (6, 'VN'),
    (7, 'MX'),
    (8, 'EZ'),
    (9, 'PTS'),
    (10, 'MKK BD'),
    (11, 'Eqvatech'),
    (12, 'PTS Autocredit')
ON CONFLICT (id) DO NOTHING;

-- Skip if a service with this 'id' already exists
COMMIT;