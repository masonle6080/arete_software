import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { announceTreeChange } from "../../../shell/treeChanges";
import { Button, EmptyText } from "../../../ui";
import { styleApi } from "../api";
import { canBeCover } from "../cover";
import type { StyleFile } from "../types";
import type { StyleSection, StyleSectionProps } from ".";

/**
 * Flat sketches and other design files attached to the Style. An image file
 * can be set as the Style's Cover; the header then shows it.
 */
function FilesSectionView({ style, onStyleChanged }: StyleSectionProps) {
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

  /**
   * Runs one change, shows its error, and reloads the Style for its Cover.
   * A change to the Cover also tells the Thesis tree and Overview, which
   * show Covers, to reload.
   */
  async function change(run: () => Promise<unknown>, changesCover: boolean) {
    setBusy(true);
    setError(undefined);
    try {
      await run();
      onStyleChanged();
      if (changesCover) announceTreeChange();
    } catch (raw) {
      setError(toCoreError(raw));
    } finally {
      setBusy(false);
    }
  }

  const coverId = style.cover?.id;

  return (
    <div className="style-files">
      {error && <p className="error">{error.message}</p>}
      {files.length === 0 ? (
        <EmptyText>No design files yet.</EmptyText>
      ) : (
        <ul>
          {files.map((f) => (
            <li key={f.id}>
              <span className="file-name" title={f.path}>
                {f.fileName}
              </span>
              {f.id === coverId ? (
                <>
                  <span className="cover-mark">Cover</span>
                  <Button size="small" disabled={busy} onClick={() => change(() => styleApi.clearStyleCover(style.id), true)}>
                    Clear Cover
                  </Button>
                </>
              ) : (
                canBeCover(f.fileName) && (
                  <Button size="small" disabled={busy} onClick={() => change(() => styleApi.setStyleCover(style.id, f.id), true)}>
                    Set as Cover
                  </Button>
                )
              )}
              <Button
                size="small"
                variant="plain"
                disabled={busy}
                onClick={() =>
                  change(async () => setFiles(await styleApi.removeStyleFile(style.id, f.id)), f.id === coverId)
                }
              >
                Remove
              </Button>
            </li>
          ))}
        </ul>
      )}
      <Button onClick={attach} disabled={busy}>
        Attach design files
      </Button>
    </div>
  );
}

export const filesSection: StyleSection = { id: "files", title: "Design files", segment: "design", Component: FilesSectionView };
