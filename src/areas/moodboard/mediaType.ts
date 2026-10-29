/** Maps between image MIME types and media file extensions. */

const byExtension: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  svg: "image/svg+xml",
  avif: "image/avif",
  bmp: "image/bmp",
};

const byMime: Record<string, string> = {
  "image/png": "png",
  "image/jpeg": "jpg",
  "image/gif": "gif",
  "image/webp": "webp",
  "image/svg+xml": "svg",
  "image/avif": "avif",
  "image/bmp": "bmp",
};

/**
 * The extension to store a dropped file under: its own (from Finder), else
 * one from its MIME type (browser drags often have no real name), else "".
 * Always plain lower-case ASCII, because it crosses IPC in a header.
 */
export function importExtension(file: { name: string; type: string }): string {
  const dot = file.name.lastIndexOf(".");
  const own = dot > 0 ? file.name.slice(dot + 1).toLowerCase() : "";
  if (/^[a-z0-9]+$/.test(own)) return own;
  return byMime[file.type] ?? "";
}

/** The MIME type of a stored media file, from its `<hash>.<ext>` name. */
export function mimeTypeOf(mediaName: string): string {
  const dot = mediaName.lastIndexOf(".");
  const ext = dot > 0 ? mediaName.slice(dot + 1) : "";
  return byExtension[ext] ?? "application/octet-stream";
}
