-- Passages (ADR-0001). The text owns each Passage's boundaries: the
-- Markdown file holds <!--passage:ID--> markers. This table holds what the
-- text cannot: the Passage's document, its last known text, and whether it
-- is orphaned (its text was fully deleted). A Passage row is never deleted
-- by a save, so links to it (Design Decisions, Moodboard items, Clips)
-- survive. `id` is the ID in the markers (a UUID from the document module).
CREATE TABLE passage (
    id              TEXT PRIMARY KEY NOT NULL,
    document_id     INTEGER NOT NULL REFERENCES document (id) ON DELETE CASCADE,
    last_known_text TEXT NOT NULL,
    orphaned        INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX passage_document ON passage (document_id);
