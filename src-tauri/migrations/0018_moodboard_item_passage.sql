-- Moodboard items linked to Passages (spec #1, ticket #12).
-- The scene JSON owns the link: a canvas element carries
-- `customData.passageId` (ADR-0003). This table is a copy of those links
-- that the core rebuilds on every scene save, so the core can answer
-- "which Moodboard items link to Passage X" without reading every scene.
-- Never write it except from `save_moodboard_scene`.
-- `element_type`, `text`, `media`, and `color` describe the item for lists
-- (the Excalidraw element type, its text, its image MediaRef, its fill).
-- A Passage row is never deleted by a document save, so an orphaned
-- Passage keeps its links. Deleting the document or Moodboard removes them.
CREATE TABLE moodboard_item_passage (
    moodboard_id INTEGER NOT NULL REFERENCES moodboard (id) ON DELETE CASCADE,
    element_id   TEXT NOT NULL,
    passage_id   TEXT NOT NULL REFERENCES passage (id) ON DELETE CASCADE,
    element_type TEXT NOT NULL,
    text         TEXT,
    media        TEXT,
    color        TEXT,
    PRIMARY KEY (moodboard_id, element_id)
);

CREATE INDEX moodboard_item_passage_passage ON moodboard_item_passage (passage_id);
