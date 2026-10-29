-- Production Order: a committed order to one Vendor for a Drop's Styles
-- (GLOSSARY.md). Status values match `ProductionOrderStatus` in
-- src/production_order/status.rs. `delivery_date` is the Vendor's delivery
-- date as YYYY-MM-DD, the only planning date in the library.
-- `currency` is the home currency that the frozen unit costs are in; it is
-- set when the order is Placed.
CREATE TABLE production_order (
    id            INTEGER PRIMARY KEY,
    drop_id       INTEGER NOT NULL REFERENCES drop_ (id) ON DELETE CASCADE,
    vendor_id     INTEGER NOT NULL REFERENCES vendor (id),
    status        TEXT NOT NULL DEFAULT 'draft'
                  CHECK (status IN ('draft', 'placed', 'in_production', 'shipped', 'received', 'cancelled')),
    delivery_date TEXT CHECK (delivery_date IS NULL OR length(delivery_date) = 10),
    currency      TEXT CHECK (currency IS NULL OR length(currency) = 3),
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX production_order_by_drop ON production_order (drop_id, id);

-- The ordered quantity of one size of one Style. A missing row is zero.
CREATE TABLE production_order_quantity (
    order_id INTEGER NOT NULL REFERENCES production_order (id) ON DELETE CASCADE,
    style_id INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    size     TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    PRIMARY KEY (order_id, style_id, size)
);

-- The unit cost of each Style on the order, frozen from its Cost Sheet when
-- the order is Placed. `unit_cost_cents` is in cents of the order's
-- `currency`; `tier_quantity` is the Quantity Tier it came from.
CREATE TABLE production_order_cost (
    order_id        INTEGER NOT NULL REFERENCES production_order (id) ON DELETE CASCADE,
    style_id        INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    tier_quantity   INTEGER NOT NULL CHECK (tier_quantity > 0),
    unit_cost_cents INTEGER NOT NULL CHECK (unit_cost_cents >= 0),
    PRIMARY KEY (order_id, style_id)
);

-- A Production Order's notes go with it (see 0007_note.sql).
CREATE TRIGGER production_order_deletes_its_notes AFTER DELETE ON production_order
BEGIN
    DELETE FROM note WHERE owner_kind = 'production_order' AND owner_id = OLD.id;
END;
