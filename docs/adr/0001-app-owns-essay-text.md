# The app owns essay text

Essays are written and stored inside the app, not in Obsidian, Google Docs, or another external editor. Design Decisions link to Passages (spans of Essay text), and those links stay stable only if the app controls every edit to the text. Essays are stored in a Markdown-based format so the writing stays portable and can be exported.

## Considered Options

- **Write in an external tool and import or link**: rejected. External edits can move or delete text without the app knowing, which breaks Passage links silently.
