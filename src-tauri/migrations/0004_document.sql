-- Long-form documents (Essays, later Scripts). The text lives in
-- documents/<file_name> as Markdown (ADR-0001). This table holds the record.
CREATE TABLE document (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    kind       TEXT NOT NULL,
    title      TEXT NOT NULL,
    file_name  TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- Increases on every create and save. Orders lists by last edit even when
    -- two edits share a timestamp.
    edit_seq   INTEGER NOT NULL
);

CREATE INDEX document_kind_edit ON document (kind, edit_seq);
