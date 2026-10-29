-- Tech Pack: the full specification for a Style that is not built on a
-- Blank (GLOSSARY: Tech Pack, Point of Measure). A Style has at most one
-- Tech Pack, keyed by the Style. The Style's base is the Tech Pack only
-- while `style_base.tech_pack_id` (below) points to it. A Style that moves
-- to a Blank keeps its Tech Pack row, so moving back restores it.
CREATE TABLE tech_pack (
    style_id           INTEGER PRIMARY KEY REFERENCES style (id) ON DELETE CASCADE,
    -- The unit of every Point of Measure value and tolerance, for example "cm".
    unit               TEXT NOT NULL DEFAULT 'cm',
    fabric_composition TEXT NOT NULL DEFAULT '',
    fabric_weight      TEXT NOT NULL DEFAULT '',
    fabric_color       TEXT NOT NULL DEFAULT '',
    fabric_vendor_id   INTEGER REFERENCES vendor (id),
    trims              TEXT NOT NULL DEFAULT '',
    construction_notes TEXT NOT NULL DEFAULT '',
    label_placement    TEXT NOT NULL DEFAULT '',
    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- The sizes of a Tech Pack: the columns of its Point of Measure table, in
-- the order the founder added them. The ID is stable.
CREATE TABLE tech_pack_size (
    id       INTEGER PRIMARY KEY,
    style_id INTEGER NOT NULL REFERENCES tech_pack (style_id) ON DELETE CASCADE,
    name     TEXT NOT NULL CHECK (trim(name) <> ''),
    UNIQUE (style_id, name)
);

-- The Points of Measure of a Tech Pack: the rows of its table, in the order
-- the founder added them. The ID is stable; Tech Pack exports and fit notes
-- refer to it.
CREATE TABLE tech_pack_pom (
    id        INTEGER PRIMARY KEY,
    style_id  INTEGER NOT NULL REFERENCES tech_pack (style_id) ON DELETE CASCADE,
    name      TEXT NOT NULL CHECK (trim(name) <> ''),
    tolerance REAL NOT NULL DEFAULT 0 CHECK (tolerance >= 0)
);

CREATE INDEX tech_pack_pom_by_style ON tech_pack_pom (style_id, id);

-- One cell of the table: the value of a Point of Measure in one size. A
-- missing row is an empty cell.
CREATE TABLE tech_pack_pom_value (
    pom_id  INTEGER NOT NULL REFERENCES tech_pack_pom (id) ON DELETE CASCADE,
    size_id INTEGER NOT NULL REFERENCES tech_pack_size (id) ON DELETE CASCADE,
    value   REAL NOT NULL,
    PRIMARY KEY (pom_id, size_id)
);

-- Flat drawings of a Tech Pack, in the order attached. `media` is a MediaRef.
CREATE TABLE tech_pack_flat (
    id        INTEGER PRIMARY KEY,
    style_id  INTEGER NOT NULL REFERENCES tech_pack (style_id) ON DELETE CASCADE,
    media     TEXT NOT NULL,
    file_name TEXT NOT NULL,
    added_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Style base (spec #1): exactly one of a Blank or a Tech Pack. This adds the
-- Tech Pack reference beside `blank_id` (0015_blank.sql). The core sets
-- exactly one of the two on every row (see src/style/base.rs).
ALTER TABLE style_base ADD COLUMN tech_pack_id INTEGER REFERENCES tech_pack (style_id);
