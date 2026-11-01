/**
 * The file kinds that can be a Style's Cover: PNG, JPEG, WebP, and HEIC.
 * Mirrors `COVER_EXTENSIONS` in `src-tauri/src/style/mod.rs`. The core
 * enforces the rule; the UI uses this list only to offer "Set as Cover".
 */
const coverExtensions = ["png", "jpg", "jpeg", "webp", "heic"];

/** Whether a design file with this name can be a Cover. */
export function canBeCover(fileName: string): boolean {
  const dot = fileName.lastIndexOf(".");
  return dot > 0 && coverExtensions.includes(fileName.slice(dot + 1).toLowerCase());
}
