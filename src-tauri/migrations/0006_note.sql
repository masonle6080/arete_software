-- Notes: a shared, dated note-and-attachment record (spec #1, "Notes"). A
-- note belongs to one owner record, named by kind and ID. Kinds cover every
-- planned owner now, so Samples and Production Orders reuse this table
-- without a schema change. owner_id has no foreign key because it points at
-- a different table per kind; the Library core checks that the owner exists.
CREATE TABLE note (
    id         INTEGER PRIMARY KEY,
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('vendor', 'sample', 'production_order')),
    owner_id   INTEGER NOT NULL,
    body       TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX note_by_owner ON note (owner_kind, owner_id, created_at);

-- The files attached to a note, in the order the founder added them. `media`
-- is a MediaRef (a file name in <library>/media/); `file_name` is the name
-- the file had when it was imported.
CREATE TABLE note_attachment (
    note_id   INTEGER NOT NULL REFERENCES note (id) ON DELETE CASCADE,
    position  INTEGER NOT NULL,
    media     TEXT NOT NULL,
    file_name TEXT NOT NULL,
    PRIMARY KEY (note_id, position)
);

-- A Vendor's notes go with it. The migrations that add Samples and
-- Production Orders add the same trigger for their tables.
CREATE TRIGGER vendor_deletes_its_notes AFTER DELETE ON vendor
BEGIN
    DELETE FROM note WHERE owner_kind = 'vendor' AND owner_id = OLD.id;
END;
