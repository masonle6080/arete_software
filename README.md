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

## Layout

- `src-tauri/src/<area>/`: Library core, one module per domain area
  (`library`, later `thesis`, `essay`, ...). Each area exposes plain domain
  command functions. All rules live here.
- `src-tauri/src/ipc/<area>.rs`: thin Tauri commands over those functions.
  Register them in `src-tauri/src/ipc/mod.rs`.
- `src-tauri/migrations/NNNN_name.sql`: one file per schema migration, applied
  in file name order on library open. Add a new file; never edit a shipped one.
- `src-tauri/tests/<area>.rs`: integration tests. They call domain functions
  against a fresh temporary library (`tests/common::TestEnv`) and check results
  only through the same API.
- `src/areas/<area>/`: UI for one area. Register it in `src/areas/index.ts`.
- `src/shell/`: app shell, navigation, and the IPC helper `call()`.
