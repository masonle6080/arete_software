# A workspace centered on one current Thesis

The sidebar is built around one current context, picked from a dropdown at its top. The context is either a Thesis or a **Shared** area. Records that exist apart from any one Thesis (Essays, Vendors, Blanks, Clips) are the Shared areas, listed in the same dropdown after the Theses. A Thesis is the one record for an idea and its release of clothing, and the founder works on one Thesis at a time, so the Thesis is the natural frame for almost every task.

In Thesis context, the sidebar shows six tab rows: Overview, Essays, Moodboards, Styles, Documentary and Production. ⌘1–⌘6 open them in that order, which follows how an idea becomes clothing. In Shared context, the sidebar shows one row for each Shared area. The sidebar shows no rows for single items: Essays, Styles and Moodboards appear only on their tab's page. The open row is always a tab row or a Shared area row. An open item marks the tab that owns it (a Style, or a Style's Moodboard, marks Styles).

There is no Drop record. "Drop" names only the release phase of a Thesis (Design → Released). Styles and Production Orders belong directly to a Thesis.

## Considered Options

- **One sidebar item for each area** (Theses, Essays, Styles, Vendors, ...): rejected. It is simple to extend, but each screen stands alone, so the founder cannot see which Thesis a Style belongs to or which Thesis an Essay supports.
- **Every Thesis as a top-level row in the tree**: rejected. The founder preferred one dropdown for the current Thesis only. "New Thesis…" and "All Theses…" sit at the end of the dropdown.
- **A source-list tree of the Thesis with item rows** (the Drop, its Styles, each Essay and Moodboard): replaced. The item rows made the sidebar long and unstable, and the separate Drop level added hierarchy without meaning. The founder never thinks about a Drop apart from its Thesis. Six fixed tab rows keep the sidebar short, and the page shows the items.
- **A Drop as its own record under a Thesis**: replaced. It added a second name, Cancelled Drops and a "create a Drop" step. A Thesis now has one lifecycle that includes the release phases.

## Consequences

- The app remembers the current Thesis for each Library and opens it at launch.
- An Essay opened from a Thesis stays in that Thesis's context. A new Essay made there links to the Thesis automatically. The same Essay opened from Shared has no Thesis context.
- One typed list of Thesis tabs drives the sidebar rows, the breadcrumb labels and the ⌘1–⌘6 shortcuts. A new part of a Thesis is a new tab in that list. A new library-wide area goes under Shared.
- The Thesis area is not a plain `Area`: the shell owns its navigation.
- A chip that links to an item in another Thesis makes that Thesis current, so the sidebar always matches the open item.
- The sidebar does not need to reload when a Style, Essay or Moodboard is added, renamed or removed, because it shows no item rows.
- The breadcrumb at the top of the content pane shows the full path, and ⌘[ goes back one level.
