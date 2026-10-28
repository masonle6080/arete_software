-- Moodboard: a freeform canvas attached to a Thesis or a single Style
-- (GLOSSARY.md). The owner is generic: `owner_kind` matches
-- `MoodboardOwner` in src/moodboard/mod.rs, and the core checks that the
-- owner exists. No CHECK on `owner_kind`, so a new owner kind needs no table
-- rebuild.
-- `scene` is the canvas state as JSON (Excalidraw "database" export, ADR-0003).
-- Images are not inlined; elements reference media by MediaRef.
CREATE TABLE moodboard (
    id         INTEGER PRIMARY KEY,
    owner_kind TEXT NOT NULL,
    owner_id   INTEGER NOT NULL,
    title      TEXT NOT NULL,
    scene      TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX moodboard_owner ON moodboard (owner_kind, owner_id);
