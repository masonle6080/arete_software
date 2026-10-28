-- Essay <-> Thesis links (many-to-many). An Essay with no row here supports
-- no Thesis, which is valid (GLOSSARY.md).
CREATE TABLE essay_thesis (
    document_id INTEGER NOT NULL REFERENCES document (id) ON DELETE CASCADE,
    thesis_id   INTEGER NOT NULL REFERENCES thesis (id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, thesis_id)
);

CREATE INDEX essay_thesis_thesis ON essay_thesis (thesis_id);
