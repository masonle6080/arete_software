-- Merge Drop into Thesis (spec #44, ticket #46). The separate Drop record goes
-- away; a Drop is now the release phase of a Thesis, not a record. Styles and Production Orders belong to a Thesis, and the Thesis
-- status covers the release (Design → Released). Status values match
-- `ThesisStatus` in src/thesis/status.rs.
--
-- This is a clean reset: every record in the library is deleted. Library
-- settings (`library_setting`) and exchange rates stay. Stored media files
-- stay on disk; no record points at them any more.

DELETE FROM production_order_cost;
DELETE FROM production_order_quantity;
DELETE FROM production_order;
DELETE FROM cost_line_price;
DELETE FROM cost_line;
DELETE FROM cost_tier;
DELETE FROM sample_fit;
DELETE FROM sample_photo;
DELETE FROM sample;
DELETE FROM decoration_color;
DELETE FROM design_decision_passage;
DELETE FROM design_decision;
DELETE FROM decoration;
DELETE FROM tech_pack_flat;
DELETE FROM tech_pack_pom_value;
DELETE FROM tech_pack_pom;
DELETE FROM tech_pack_size;
DELETE FROM style_size;
DELETE FROM style_base;
DELETE FROM tech_pack;
UPDATE style SET cover_file_id = NULL;
DELETE FROM style_file;
DELETE FROM style;
DELETE FROM drop_;
DELETE FROM moodboard_item_passage;
DELETE FROM moodboard;
DELETE FROM clip_passage;
DELETE FROM clip_tag;
DELETE FROM documentary_clip;
DELETE FROM documentary;
DELETE FROM essay_thesis;
DELETE FROM passage;
DELETE FROM document;
DELETE FROM clip;
DELETE FROM blank_size;
DELETE FROM blank_color;
DELETE FROM blank;
DELETE FROM note_attachment;
DELETE FROM note;
DELETE FROM vendor_contact;
DELETE FROM vendor_role;
DELETE FROM vendor;
DELETE FROM thesis;

-- The tables whose columns or checks change are made again. Other tables
-- refer to them by name, so their foreign keys bind to the new tables.
DROP TABLE production_order;
DROP TABLE style;
DROP TABLE drop_;
DROP TABLE thesis;

CREATE TABLE thesis (
    id         INTEGER PRIMARY KEY,
    title      TEXT NOT NULL,
    statement  TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'seed'
               CHECK (status IN ('seed', 'developing', 'design', 'sampling', 'production',
                                 'released', 'archived', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Style: a single garment design in a single color, on one Thesis.
-- `cover_file_id` is its Cover (see 0025_style_cover.sql).
CREATE TABLE style (
    id            INTEGER PRIMARY KEY,
    thesis_id     INTEGER NOT NULL REFERENCES thesis (id),
    name          TEXT NOT NULL,
    color         TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'draft'
                  CHECK (status IN ('draft', 'sampling', 'approved', 'cut')),
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    cover_file_id INTEGER REFERENCES style_file (id) ON DELETE SET NULL
);

CREATE INDEX style_by_thesis ON style (thesis_id);

-- Production Order: a committed order to one Vendor for a Thesis's Styles.
-- The other columns are as in 0024_production_order.sql.
CREATE TABLE production_order (
    id            INTEGER PRIMARY KEY,
    thesis_id     INTEGER NOT NULL REFERENCES thesis (id) ON DELETE CASCADE,
    vendor_id     INTEGER NOT NULL REFERENCES vendor (id),
    status        TEXT NOT NULL DEFAULT 'draft'
                  CHECK (status IN ('draft', 'placed', 'in_production', 'shipped', 'received', 'cancelled')),
    delivery_date TEXT CHECK (delivery_date IS NULL OR length(delivery_date) = 10),
    currency      TEXT CHECK (currency IS NULL OR length(currency) = 3),
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX production_order_by_thesis ON production_order (thesis_id, id);

-- Dropping the table dropped its trigger (see 0024_production_order.sql).
CREATE TRIGGER production_order_deletes_its_notes AFTER DELETE ON production_order
BEGIN
    DELETE FROM note WHERE owner_kind = 'production_order' AND owner_id = OLD.id;
END;
