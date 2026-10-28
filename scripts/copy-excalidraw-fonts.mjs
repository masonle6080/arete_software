// Copies the Excalidraw fonts into public/fonts so the Moodboard canvas works
// offline (ADR-0002). By default Excalidraw loads fonts from a CDN.
// index.html sets window.EXCALIDRAW_ASSET_PATH = "/" to point at this copy.
// Runs on `npm install` (postinstall) and before dev and build.
import { cpSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const from = `${root}node_modules/@excalidraw/excalidraw/dist/prod/fonts`;
const to = `${root}public/fonts`;

if (!existsSync(from)) {
  console.error(`copy-excalidraw-fonts: ${from} is missing; run npm install`);
  process.exit(1);
}
cpSync(from, to, { recursive: true });
