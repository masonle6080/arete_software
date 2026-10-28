-- Vendor details (GLOSSARY: Vendor): location, minimum order quantity, typical
-- lead time (free text), and contact people.
ALTER TABLE vendor ADD COLUMN location TEXT NOT NULL DEFAULT '';
ALTER TABLE vendor ADD COLUMN minimum_order_quantity INTEGER
    CHECK (minimum_order_quantity IS NULL OR minimum_order_quantity > 0);
ALTER TABLE vendor ADD COLUMN lead_time TEXT NOT NULL DEFAULT '';

-- The contact people at a Vendor, in the order the founder lists them.
CREATE TABLE vendor_contact (
    vendor_id INTEGER NOT NULL REFERENCES vendor (id) ON DELETE CASCADE,
    position  INTEGER NOT NULL,
    name      TEXT NOT NULL CHECK (trim(name) <> ''),
    role      TEXT NOT NULL DEFAULT '',
    email     TEXT NOT NULL DEFAULT '',
    phone     TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (vendor_id, position)
);
