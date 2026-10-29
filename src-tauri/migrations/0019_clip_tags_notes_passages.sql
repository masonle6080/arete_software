-- Clip tags, notes, and Passage links (spec #1, ticket #25).

-- Free-text notes on a Clip.
ALTER TABLE clip ADD COLUMN notes TEXT NOT NULL DEFAULT '';

-- A Clip's tags. A tag is trimmed, non-empty text; case does not make two
-- tags different ("Beach" and "beach" are one tag).
CREATE TABLE clip_tag (
    clip_id INTEGER NOT NULL REFERENCES clip (id) ON DELETE CASCADE,
    tag     TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY (clip_id, tag)
);
CREATE INDEX clip_tag_by_tag ON clip_tag (tag);

-- Clip ↔ Passage is many-to-many. A Passage row is never deleted by a save
-- (ADR-0001), so links survive while its text is orphaned.
CREATE TABLE clip_passage (
    clip_id    INTEGER NOT NULL REFERENCES clip (id) ON DELETE CASCADE,
    passage_id TEXT NOT NULL REFERENCES passage (id) ON DELETE CASCADE,
    linked_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (clip_id, passage_id)
);
CREATE INDEX clip_passage_by_passage ON clip_passage (passage_id);
