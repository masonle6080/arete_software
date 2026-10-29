-- Style: a single garment design in a single color, on one Drop
-- (GLOSSARY.md). Status values match `StyleStatus` in src/style/status.rs.
-- A Cut Style stays in the table and stays visible.
CREATE TABLE style (
    id         INTEGER PRIMARY KEY,
    drop_id    INTEGER NOT NULL REFERENCES drop_ (id),
    name       TEXT NOT NULL,
    color      TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'draft'
               CHECK (status IN ('draft', 'sampling', 'approved', 'cut')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX style_by_drop ON style (drop_id);

-- Design files (flat sketches, Illustrator or Procreate work) attached to a
-- Style, in the order the founder added them. `media` is a MediaRef (a file
-- name in <library>/media/); `file_name` is the name the file had when it
-- was imported.
CREATE TABLE style_file (
    id        INTEGER PRIMARY KEY,
    style_id  INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    media     TEXT NOT NULL,
    file_name TEXT NOT NULL,
    added_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX style_file_by_style ON style_file (style_id, id);
