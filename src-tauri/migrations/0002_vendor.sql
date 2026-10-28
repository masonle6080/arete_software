-- Vendor: a library-wide company record (GLOSSARY: Vendor). Later migrations
-- add contacts, location, minimum order quantity, lead time, and notes.
CREATE TABLE vendor (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL CHECK (trim(name) <> ''),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- The roles a Vendor fills. A Vendor has one or more roles.
CREATE TABLE vendor_role (
    vendor_id INTEGER NOT NULL REFERENCES vendor (id) ON DELETE CASCADE,
    role      TEXT NOT NULL CHECK (role IN ('blank_supplier', 'decorator', 'manufacturer', 'mill')),
    PRIMARY KEY (vendor_id, role)
);
