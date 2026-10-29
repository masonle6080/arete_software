# arete_software

Arete is a Mac desktop app (Tauri v2) for a concept-led clothing brand. See
`GLOSSARY.md` for terms and `docs/adr/` for decisions.

## Develop

Requirements: Rust (stable), Node 20.19+ and npm.

```sh
npm install                       # once
npm run tauri dev                 # run the app with hot reload
cd src-tauri && cargo test        # Library core integration tests (Rust)
npm test                          # TypeScript unit tests (Vitest)
npm run build                     # type-check and build the webview
```

## ffmpeg (Clips)

The Clip pool uses `ffprobe` to read duration and date shot and `ffmpeg` to
make thumbnails. Both run locally; nothing uses the network.

In development, the app finds them in this order: `ARETE_FFMPEG` and
`ARETE_FFPROBE`, next to the app executable, `/opt/homebrew/bin`,
`/usr/local/bin`, then `PATH`. Install them with `brew install ffmpeg`. The
Clip tests that need ffmpeg skip when it is not found.

A release build bundles both as Tauri sidecars. The repo does not hold the
binaries (`src-tauri/binaries/` is ignored). To make a release build:

1. Get static macOS builds of `ffmpeg` and `ffprobe` (no Homebrew dylibs).
   Do not copy Homebrew's ffmpeg: it links to Homebrew libraries and breaks on
   other Macs. Check the license terms (GPL or LGPL) of the build you ship.
2. Copy them into `src-tauri/binaries/` with the target triple as a suffix
   (`rustc --print host-tuple` prints it):

   ```sh
   mkdir -p src-tauri/binaries
   cp /path/to/ffmpeg  src-tauri/binaries/ffmpeg-aarch64-apple-darwin
   cp /path/to/ffprobe src-tauri/binaries/ffprobe-aarch64-apple-darwin
   chmod +x src-tauri/binaries/*
   ```

   For an Intel or universal build, also add the `-x86_64-apple-darwin` files.
3. Check that each one is self-contained: `otool -L <file>` must list only
   system libraries (`/usr/lib/...`, `/System/...`).
4. Build with the release config, which adds `bundle.externalBin`:

   ```sh
   npm run tauri build -- --config src-tauri/tauri.release.conf.json
   ```

Tauri puts the sidecars in `Arete.app/Contents/MacOS/`, next to the app
executable, where the app finds them.

## Layout

- `src-tauri/src/<area>/`: Library core, one module per domain area
  (`library`, later `thesis`, `essay`, ...). Each area exposes plain domain
  command functions. All rules live here. Every database write goes through
  `Library::writer()`, which refuses with `LibraryDisconnected` when the
  library folder is unreachable. File writes call `Library::ensure_reachable()`
  first.
- `src-tauri/src/ipc/<area>.rs`: thin Tauri commands over those functions.
  Register them in `src-tauri/src/ipc/mod.rs`.
- `src-tauri/migrations/NNNN_name.sql`: one file per schema migration, applied
  in file name order on library open. Add a new file; never edit a shipped one.
- `src-tauri/tests/<area>.rs`: integration tests. They call domain functions
  against a fresh temporary library (`tests/common::TestEnv`) and check results
  only through the same API.
- `src/areas/<area>/`: UI for one area. Register it in `src/areas/index.ts`.
- `src/document/`: the document module (Markdown ⇄ editor document) and the
  shared editor, for Essays and Scripts. Vitest tests sit next to it.
- `src/shell/`: app shell, navigation, and the IPC helper `call()`.
- `src/export/`: shared exports. `exportPdf({ title, html, css })` opens a
  print window (`#print=ID`) and the system print dialog ("Save as PDF").
  `saveTextFile(...)` asks where to save and writes outside the library.
  Every PDF export (Essay, Tech Pack, Style trace) goes through `exportPdf`.
