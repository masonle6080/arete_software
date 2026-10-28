-- Documentary: a film that develops one Thesis (GLOSSARY.md). A Thesis has
-- at most one. Status values match `DocumentaryStatus` in
-- src/documentary/status.rs.
CREATE TABLE documentary (
    id         INTEGER PRIMARY KEY,
    thesis_id  INTEGER NOT NULL UNIQUE REFERENCES thesis (id),
    title      TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'planning'
               CHECK (status IN ('planning', 'filming', 'editing', 'released', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Clip ↔ Documentary is many-to-many.
CREATE TABLE documentary_clip (
    documentary_id INTEGER NOT NULL REFERENCES documentary (id),
    clip_id        INTEGER NOT NULL REFERENCES clip (id),
    added_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (documentary_id, clip_id)
);
CREATE INDEX documentary_clip_by_clip ON documentary_clip (clip_id);
