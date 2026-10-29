-- Sample: a physical piece that a Vendor makes for a Style before production
-- (GLOSSARY.md). Each Sample is one numbered round for its Style and Vendor.
-- Outcome values match `SampleOutcome` in src/sample/mod.rs. A revised
-- Sample leads to the next round.
CREATE TABLE sample (
    id         INTEGER PRIMARY KEY,
    style_id   INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    vendor_id  INTEGER NOT NULL REFERENCES vendor (id),
    round      INTEGER NOT NULL CHECK (round >= 1),
    outcome    TEXT NOT NULL DEFAULT 'pending'
               CHECK (outcome IN ('pending', 'approved', 'revised', 'rejected')),
    feedback   TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (style_id, vendor_id, round)
);

CREATE INDEX sample_by_style ON sample (style_id, id);

-- Photos of a Sample, in the order the founder added them. `media` is a
-- MediaRef (a file name in <library>/media/); `file_name` is the name the
-- file had when it was imported.
CREATE TABLE sample_photo (
    id        INTEGER PRIMARY KEY,
    sample_id INTEGER NOT NULL REFERENCES sample (id) ON DELETE CASCADE,
    media     TEXT NOT NULL,
    file_name TEXT NOT NULL,
    added_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX sample_photo_by_sample ON sample_photo (sample_id, id);

-- A Sample's notes go with it (see 0007_note.sql).
CREATE TRIGGER sample_deletes_its_notes AFTER DELETE ON sample
BEGIN
    DELETE FROM note WHERE owner_kind = 'sample' AND owner_id = OLD.id;
END;
