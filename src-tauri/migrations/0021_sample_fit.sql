-- Fit notes (GLOSSARY: Fit Note): the measured value of one Point of
-- Measure, in one size, on a Sample of a Tech Pack Style. A missing row is a
-- value not yet measured.
--
-- The references to the Point of Measure and the size have no ON DELETE
-- action (NO ACTION), so the database refuses to remove a Point of Measure
-- or a size while a fit note refers to it: a recorded measurement is never
-- lost silently. The core says so first (src/tech_pack/mod.rs). Removing the
-- whole Style still works, because its Samples, and so their fit notes, go
-- in the same statement.
CREATE TABLE sample_fit (
    sample_id INTEGER NOT NULL REFERENCES sample (id) ON DELETE CASCADE,
    pom_id    INTEGER NOT NULL REFERENCES tech_pack_pom (id),
    size_id   INTEGER NOT NULL REFERENCES tech_pack_size (id),
    measured  REAL NOT NULL,
    PRIMARY KEY (sample_id, pom_id, size_id)
);

CREATE INDEX sample_fit_by_pom ON sample_fit (pom_id);
CREATE INDEX sample_fit_by_size ON sample_fit (size_id);
