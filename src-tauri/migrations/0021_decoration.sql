-- Decoration: a treatment applied to a Style's base, such as a print,
-- embroidery, dye, or label (GLOSSARY.md). `artwork_media` is a MediaRef (a
-- file name in <library>/media/) and `artwork_file_name` is the name the
-- file had when it was imported; both are NULL when there is no artwork.
-- `vendor_id` is the Vendor that applies it; NULL when not chosen yet.
CREATE TABLE decoration (
    id                INTEGER PRIMARY KEY,
    style_id          INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    technique         TEXT NOT NULL CHECK (trim(technique) <> ''),
    placement         TEXT NOT NULL DEFAULT '',
    size              TEXT NOT NULL DEFAULT '',
    artwork_media     TEXT,
    artwork_file_name TEXT,
    vendor_id         INTEGER REFERENCES vendor (id),
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK ((artwork_media IS NULL) = (artwork_file_name IS NULL))
);

CREATE INDEX decoration_by_style ON decoration (style_id, id);

-- The colors of a Decoration, in the order the founder lists them.
CREATE TABLE decoration_color (
    decoration_id INTEGER NOT NULL REFERENCES decoration (id) ON DELETE CASCADE,
    position      INTEGER NOT NULL,
    color         TEXT NOT NULL CHECK (trim(color) <> ''),
    PRIMARY KEY (decoration_id, position)
);

-- A Design Decision's optional link to one Decoration. The core keeps the
-- Decoration on the same Style as the Design Decision. Removing the
-- Decoration clears the link and keeps the Design Decision.
ALTER TABLE design_decision ADD COLUMN decoration_id INTEGER REFERENCES decoration (id) ON DELETE SET NULL;

CREATE INDEX design_decision_by_decoration ON design_decision (decoration_id);
