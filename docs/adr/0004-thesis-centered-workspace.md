# A workspace centered on one current Thesis

The sidebar is built around one current Thesis, picked from a dropdown at its top. Under the dropdown, a source-list tree shows that Thesis: Overview, Essays, Moodboards, the Drop with its Styles, and the Documentary. Records that exist apart from any one Thesis (Essays, Vendors, Blanks, Clips) sit under a **Shared** heading. Each Drop expresses one Thesis, and the founder works on one Thesis at a time, so the Thesis is the natural frame for almost every task. Showing its contents as one tree also makes the link from idea to garment visible, and that link was the main thing the old layout hid.

## Considered Options

- **One sidebar item for each area** (Theses, Essays, Styles, Vendors, ...): rejected. It is simple to extend, but each screen stands alone, so the founder cannot see which Drop a Style belongs to or which Thesis an Essay supports.
- **Every Thesis as a top-level row in the tree**: rejected. The founder preferred one dropdown and a tree for the current Thesis only. "New Thesis…" and "All Theses…" sit at the end of the dropdown.
- **A tree that goes below Style** (Moodboard, Samples): rejected. The tree stops at Style, and the Style screen's Design / Build / Make segments open the deeper parts. A breadcrumb shows the full path.

## Consequences

- The app remembers the current Thesis for each Library and opens it at launch.
- An Essay opened from a Thesis stays in that Thesis's context. A new Essay made there links to the Thesis automatically. The same Essay opened from Shared has no Thesis context.
- The Thesis area is not a plain `Area`: the shell owns its navigation. A new library-wide area goes under Shared. A new part of a Thesis is a new Thesis section.
- The tree must reload when a Style, Essay, or Moodboard of the current Thesis is added, renamed, or removed.
