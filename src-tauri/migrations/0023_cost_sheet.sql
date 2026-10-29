-- Cost Sheet: the itemized unit cost of a Style at each Quantity Tier
-- (GLOSSARY.md). Money is exact: a unit cost is an INTEGER in ten-thousandths
-- of its line's quote currency, and an exchange rate is an INTEGER in
-- millionths. See src/cost_sheet/money.rs.

-- The Quantity Tiers of a Style's Cost Sheet. They show in quantity order.
CREATE TABLE cost_tier (
    id       INTEGER PRIMARY KEY,
    style_id INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    UNIQUE (style_id, quantity)
);

-- The lines of a Style's Cost Sheet, in the order the founder added them.
-- Category values match `CostCategory` in src/cost_sheet/mod.rs. `currency`
-- is the Vendor's quote currency (an ISO 4217 code such as USD).
CREATE TABLE cost_line (
    id          INTEGER PRIMARY KEY,
    style_id    INTEGER NOT NULL REFERENCES style (id) ON DELETE CASCADE,
    category    TEXT NOT NULL CHECK (category IN
                    ('base', 'decoration', 'trims', 'labels', 'packaging', 'shipping', 'duties', 'other')),
    vendor_id   INTEGER REFERENCES vendor (id),
    description TEXT NOT NULL DEFAULT '',
    currency    TEXT NOT NULL CHECK (length(currency) = 3)
);

CREATE INDEX cost_line_by_style ON cost_line (style_id, id);

-- The unit cost of one line at one Quantity Tier, in ten-thousandths of the
-- line's currency. A missing row is a blank cell.
CREATE TABLE cost_line_price (
    line_id   INTEGER NOT NULL REFERENCES cost_line (id) ON DELETE CASCADE,
    tier_id   INTEGER NOT NULL REFERENCES cost_tier (id) ON DELETE CASCADE,
    unit_cost INTEGER NOT NULL CHECK (unit_cost >= 0),
    PRIMARY KEY (line_id, tier_id)
);

-- Exchange rates that the founder enters by hand: one unit of `currency` is
-- worth `rate` millionths of the home currency. The home currency itself is
-- the library setting `home_currency`.
CREATE TABLE exchange_rate (
    currency TEXT PRIMARY KEY NOT NULL CHECK (length(currency) = 3),
    rate     INTEGER NOT NULL CHECK (rate > 0)
);
