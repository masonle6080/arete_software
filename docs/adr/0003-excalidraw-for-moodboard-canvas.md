# Excalidraw for the Moodboard canvas

Moodboards use Excalidraw (`@excalidraw/excalidraw`, pinned at 0.18.1) as the canvas. Excalidraw is MIT-licensed, so the app can ship it for commercial use with no license key, no watermark, and no telemetry. It already gives the canvas the founder needs: images, text, color swatches (filled rectangles), resize, and grouping. The canvas state is saved as Excalidraw "database" JSON in the library database. Images are not inlined in that JSON: each image element's `fileId` is the media name of a file imported through the media import path, so images dedup and stay in the library folder (ADR-0002).

## Considered Options

- **tldraw**: rejected. Its license is proprietary and source-available. A production app that gives functionality to end users needs a trial or commercial license key, and the library has license-key checks, a watermark, and may send usage data to tldraw. Telemetry conflicts with ADR-0002 (local-first).
- **Build our own canvas**: rejected. Placement, resize, grouping, and undo are a large amount of work that Excalidraw already does well.

## Consequences

- Excalidraw loads its fonts from a CDN by default. The app copies them into `public/fonts` at install and build time and sets `window.EXCALIDRAW_ASSET_PATH`, so the canvas works offline.
- The Tauri window sets `dragDropEnabled: false`, so the webview, not Tauri, receives HTML5 drops of images from Finder or a browser.
- The scene JSON format belongs to Excalidraw. Upgrading Excalidraw must keep old scenes loadable; Excalidraw restores old versions on load.
