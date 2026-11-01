/**
 * The current Thesis, the one the sidebar centers on. It is a UI preference,
 * not library data, so it lives in the webview's storage, one per library
 * folder.
 */
type Storage = Pick<globalThis.Storage, "getItem" | "setItem" | "removeItem">;

const key = (libraryPath: string) => `arete.currentThesis:${libraryPath}`;

export function loadCurrentThesis(libraryPath: string, storage: Storage = localStorage): number | undefined {
  const raw = storage.getItem(key(libraryPath));
  const id = Number(raw);
  return raw && Number.isInteger(id) ? id : undefined;
}

export function saveCurrentThesis(libraryPath: string, id: number | undefined, storage: Storage = localStorage) {
  if (id === undefined) storage.removeItem(key(libraryPath));
  else storage.setItem(key(libraryPath), String(id));
}
