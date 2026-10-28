-- Thesis: a philosophical idea that a Drop expresses (GLOSSARY.md).
-- Status values match `ThesisStatus` in src/thesis/status.rs.
CREATE TABLE thesis (
    id         INTEGER PRIMARY KEY,
    title      TEXT NOT NULL,
    statement  TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'seed'
               CHECK (status IN ('seed', 'developing', 'committed', 'released', 'archived', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
