import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { libraryApi } from "./api";
import type { LibraryInfo } from "./session";

interface Props {
  initialError?: CoreError;
  onOpened(library: LibraryInfo): void;
}

/** Shown when no library is open: create a new one or open an existing one. */
export function WelcomeScreen({ initialError, onOpened }: Props) {
  const [error, setError] = useState<CoreError | undefined>(initialError);
  const [busy, setBusy] = useState(false);

  async function choose(action: "create" | "open") {
    const path = await open({
      directory: true,
      title: action === "create" ? "Choose an empty folder for the new library" : "Open a library",
    });
    if (typeof path !== "string") return;
    setBusy(true);
    setError(undefined);
    try {
      const lib = action === "create" ? await libraryApi.createLibrary(path) : await libraryApi.openLibrary(path);
      onOpened(lib);
    } catch (raw) {
      setError(toCoreError(raw));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="welcome">
      <h1>Arete</h1>
      <p>All brand data lives in one library folder. It can be on an external drive.</p>
      <div className="actions">
        <button disabled={busy} onClick={() => choose("create")}>
          Create library…
        </button>
        <button disabled={busy} onClick={() => choose("open")}>
          Open library…
        </button>
      </div>
      {error && <p className="error">{error.message}</p>}
    </main>
  );
}
