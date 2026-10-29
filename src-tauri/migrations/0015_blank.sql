-- Blank: a library-wide, ready-made garment model bought from a Vendor
-- (GLOSSARY: Blank). `cost_cents` is the unit cost in minor units of
-- `currency` (free text such as "USD"); NULL when unknown.
CREATE TABLE blank (
    id         INTEGER PRIMARY KEY,
    brand      TEXT NOT NULL CHECK (trim(brand) <> ''),
    model      TEXT NOT NULL CHECK (trim(model) <> ''),
    vendor_id  INTEGER NOT NULL REFERENCES vendor (id),
    cost_cents INTEGER CHECK (cost_cents IS NULL OR cost_cents >= 0),
    currency   TEXT NOT NULL DEFAULT '',
    notes      TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- The colors a Blank comes in, in the order the founder lists them.
CREATE TABLE blank_color (
    blank_id INTEGER NOT NULL REFERENCES blank (id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    color    TEXT NOT NULL CHECK (trim(color) <> ''),
    PRIMARY KEY (blank_id, position)
);

-- The sizes a Blank comes in, in the order the founder lists them.
CREATE TABLE blank_size (
    blank_id INTEGER NOT NULL REFERENCES blank (id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    size     TEXT NOT NULL CHECK (trim(size) <> ''),
    PRIMARY KEY (blank_id, position),
    UNIQUE (blank_id, size)
);

-- Style base (spec #1): exactly one of a Blank or a Tech Pack. One row per
-- Style that has a base; no row means the base is not set yet. This
-- migration adds only the Blank reference. The Tech Pack migration adds its
-- own reference column, and the core enforces that exactly one is set.
CREATE TABLE style_base (
    style_id INTEGER PRIMARY KEY REFERENCES style (id) ON DELETE CASCADE,
    blank_id INTEGER REFERENCES blank (id)
);

CREATE INDEX style_base_by_blank ON style_base (blank_id);

-- The sizes a Style is offered in. For a Blank-based Style, each one is one
-- of the Blank's sizes (the core enforces this).
CREATE TABLE style_size (
    style_id INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    size     TEXT NOT NULL,
    PRIMARY KEY (style_id, size)
);
