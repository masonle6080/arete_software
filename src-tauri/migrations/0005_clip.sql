-- The Clip pool: every video file in the library, before or after it joins
-- a Documentary. The video and its thumbnail are media (see src/media).
CREATE TABLE clip (
    id            INTEGER PRIMARY KEY,
    media         TEXT NOT NULL UNIQUE,   -- MediaRef of the video
    original_name TEXT NOT NULL,          -- file name at import
    duration_ms   INTEGER,                -- NULL when unknown
    date_shot     TEXT,                   -- YYYY-MM-DD, NULL when unknown; editable
    thumbnail     TEXT,                   -- MediaRef of a JPEG, NULL when none
    imported_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
