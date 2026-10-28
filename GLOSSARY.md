# Arete

A creative and operations system for a solo, concept-led clothing brand. Each drop of clothing expresses one philosophical idea, which essays and a documentary develop.

## Language

### Ideas and storytelling

**Thesis**:
A philosophical idea from the founder's life that a Drop expresses. A Thesis can exist and develop before any Drop is made from it.
_Avoid_: Theme, concept, idea

**Essay**:
A long-form written piece, authored in the app, that explores one idea. An Essay can support any number of Theses, including none.
_Avoid_: Article, post, writing

**Passage**:
A selected span of text within an Essay that motivates one or more Design Decisions.
_Avoid_: Quote, excerpt, highlight

**Moodboard**:
A freeform canvas of visual references, text, and color swatches, attached to a Thesis or optionally to a single Style.
_Avoid_: Inspiration board, board

**Documentary**:
A film that develops one Thesis. A Thesis has at most one Documentary, and when it has one, the Documentary is released before the Drop.
_Avoid_: Video, episode, chapter

**Script**:
The written outline of a Documentary, authored in the same way as an Essay and able to hold Passages.
_Avoid_: Outline, treatment

**Clip**:
A single video file in the library. A Clip can exist before any Documentary and can be used in several Documentaries.
_Avoid_: Footage item, video, shot

### Products

**Drop**:
A release of clothing made from exactly one Thesis. Each Thesis has at most one Drop that is not cancelled.
_Avoid_: Collection, release, season

**Style**:
A single garment design in a single color, made in a range of sizes. The same design in another color is a separate Style.
_Avoid_: Product, item, piece, colorway, SKU

**Design Decision**:
A choice made on a Style, classified by aspect (Decoration, fabric, color, silhouette, or other), with a written rationale and, optionally, the Passages that motivate it.
_Avoid_: Design note, spec

**Blank**:
A ready-made garment model bought from a Vendor, which a Style can use as its base. A Blank can be reused across Drops.
_Avoid_: Base garment, stock garment

**Tech Pack**:
The full specification for a Style that is not built on a Blank: measurements, fabric, trims, and construction.
_Avoid_: Spec sheet, spec pack

**Point of Measure**:
One named measurement in a Tech Pack, such as chest width, with a value for each size and a tolerance.
_Avoid_: POM, measurement, spec point

**Decoration**:
A treatment applied to a Style's base, such as a print, embroidery, dye, or label.
_Avoid_: Embellishment, graphic, treatment

### Operations

**Vendor**:
A company that supplies Blanks, materials, Decoration, or manufacturing. One Vendor can fill several of these roles.
_Avoid_: Supplier, factory, manufacturer, mill

**Sample**:
A physical piece that a Vendor makes for a Style before production. It belongs to a numbered round and ends approved, revised, or rejected; a revised Sample leads to the next round.
_Avoid_: Prototype, proto, test piece

**Cost Sheet**:
The itemized unit cost of a Style (base, Decoration, trims, labels, packaging, shipping, duties) at each Quantity Tier.
_Avoid_: Costing, BOM, price breakdown

**Quantity Tier**:
An order quantity at which a Cost Sheet gives unit costs, such as 50, 100, or 300 pieces.
_Avoid_: Price break, MOQ level

**Production Order**:
A committed order to a Vendor for a Drop's Styles, with quantities per size, unit costs, and a delivery date. It is the last step the system tracks.
_Avoid_: PO, purchase order, bulk order
