import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { styleApi } from "../api";
import type { StyleFile } from "../types";
import type { StyleSection, StyleSectionProps } from ".";

/** Flat sketches and other design files attached to the Style. */
function FilesSectionView({ style }: StyleSectionProps) {
  const [files, setFiles] = useState<StyleFile[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    styleApi.styleFiles(style.id).then(setFiles, (raw) => setError(toCoreError(raw)));
  }, [style.id]);
  useEffect(reload, [reload]);

  async function attach() {
    const picked = await open({ multiple: true, title: "Attach design files" });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    setBusy(true);
    setError(undefined);
    try {
      setFiles(await styleApi.attachStyleFiles(style.id, paths));
    } catch (raw) {
      setError(toCoreError(raw));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="style-files">
      {error && <p className="error">{error.message}</p>}
      {files.length === 0 ? (
        <p className="empty">No design files yet.</p>
      ) : (
        <ul>
          {files.map((f) => (
            <li key={f.id} title={f.path}>
              {f.fileName}
            </li>
          ))}
        </ul>
      )}
      <button onClick={attach} disabled={busy}>
        Attach design files
      </button>
    </div>
  );
}

export const filesSection: StyleSection = { id: "files", title: "Design files", Component: FilesSectionView };
