-- Drop: a release of clothing made from exactly one Thesis (GLOSSARY.md).
-- Stage values match `DropStage` in src/drop/stage.rs.
CREATE TABLE drop_ (
    id         INTEGER PRIMARY KEY,
    thesis_id  INTEGER NOT NULL REFERENCES thesis (id),
    name       TEXT NOT NULL,
    stage      TEXT NOT NULL DEFAULT 'design'
               CHECK (stage IN ('design', 'sampling', 'production', 'released', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- A Thesis has at most one Drop that is not Cancelled.
CREATE UNIQUE INDEX drop_one_active_per_thesis ON drop_ (thesis_id) WHERE stage <> 'cancelled';
