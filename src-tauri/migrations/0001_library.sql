-- Library-wide key/value settings (for example: home currency, created at).
CREATE TABLE library_setting (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT INTO library_setting (key, value)
VALUES ('created_at', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
