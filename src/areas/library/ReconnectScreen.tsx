import { useEffect, useState } from "react";
import type { CoreError } from "../../shell/coreError";
import { libraryApi } from "./api";
import { waitForReconnect } from "./reconnect";
import type { LibraryInfo } from "./session";
import { Button } from "../../ui";

interface Props {
  library: LibraryInfo;
  onReconnected(library: LibraryInfo): void;
  onSwitch(): void;
}

/**
 * Shown while the open library's folder is unreachable. It polls for the
 * folder and returns to the app when the drive is back.
 */
export function ReconnectScreen({ library, onReconnected, onSwitch }: Props) {
  const [error, setError] = useState<CoreError>();

  useEffect(() => {
    const controller = new AbortController();
    waitForReconnect(libraryApi, { signal: controller.signal })
      .then((lib) => lib && onReconnected(lib))
      .catch((err: CoreError) => setError(err));
    return () => controller.abort();
  }, [onReconnected]);

  return (
    <main className="welcome">
      <h1>Library disconnected</h1>
      <p>Arete cannot reach the library folder. Nothing was saved while it was away.</p>
      <p className="path">{library.path}</p>
      <p>Connect the drive that holds this folder. Arete reconnects by itself.</p>
      {error ? <p className="error">{error.message}</p> : <p className="waiting">Waiting for the folder…</p>}
      <div className="actions">
        <Button onClick={onSwitch}>Switch library…</Button>
      </div>
    </main>
  );
}
