-- Design Decision: a choice made on a Style, with an aspect and a written
-- rationale (GLOSSARY.md). Aspect values match `DesignAspect` in
-- src/design_decision/mod.rs. A Cut Style keeps its Design Decisions.
CREATE TABLE design_decision (
    id         INTEGER PRIMARY KEY,
    style_id   INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    aspect     TEXT NOT NULL
               CHECK (aspect IN ('decoration', 'fabric', 'color', 'silhouette', 'other')),
    rationale  TEXT NOT NULL CHECK (trim(rationale) <> ''),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX design_decision_by_style ON design_decision (style_id, id);

-- The Passages that motivate a Design Decision, many-to-many. A Passage can
-- be in any document. Passage rows are never deleted by a save (see
-- 0014_passage.sql), so a link to an orphaned Passage stays.
CREATE TABLE design_decision_passage (
    id                 INTEGER PRIMARY KEY,
    design_decision_id INTEGER NOT NULL REFERENCES design_decision (id) ON DELETE CASCADE,
    passage_id         TEXT NOT NULL REFERENCES passage (id) ON DELETE CASCADE,
    UNIQUE (design_decision_id, passage_id)
);

CREATE INDEX design_decision_passage_by_passage ON design_decision_passage (passage_id);
