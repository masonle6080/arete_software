import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Area } from "../../shell/area";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { clipApi, type Clip } from "./api";
import "./clips.css";

const VIDEO_EXTENSIONS = ["mov", "mp4", "m4v", "avi", "mkv", "mts", "m2ts", "webm", "mxf", "3gp"];

/** `m:ss`, or `h:mm:ss` for an hour or more. */
function formatDuration(ms: number | null): string {
  if (ms === null) return "–";
  const total = Math.round(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
}

function ClipCard({ clip, onChange, onError }: { clip: Clip; onChange(c: Clip): void; onError(e: CoreError): void }) {
  async function setDate(value: string) {
    try {
      onChange(await clipApi.setDateShot(clip.id, value === "" ? null : value));
    } catch (raw) {
      onError(toCoreError(raw));
    }
  }

  async function reveal() {
    try {
      await clipApi.revealClip(clip.id);
    } catch (raw) {
      onError(toCoreError(raw));
    }
  }

  return (
    <li className="clip">
      {clip.thumbnailPath ? (
        <img className="thumb" src={convertFileSrc(clip.thumbnailPath)} alt="" />
      ) : (
        <div className="thumb empty">No thumbnail</div>
      )}
      <div className="name" title={clip.originalName}>
        {clip.originalName}
      </div>
      <div className="meta">
        <span>{formatDuration(clip.durationMs)}</span>
        <label>
          Shot{" "}
          <input type="date" value={clip.dateShot ?? ""} onChange={(e) => setDate(e.target.value)} />
        </label>
      </div>
      <button onClick={reveal}>Show in Finder</button>
    </li>
  );
}

function ClipsScreen() {
  const [clips, setClips] = useState<Clip[] | null>(null);
  const [error, setError] = useState<CoreError>();
  const [importing, setImporting] = useState<string | null>(null);

  useEffect(() => {
    clipApi.listClips().then(setClips, (raw) => setError(toCoreError(raw)));
  }, []);

  async function importVideo() {
    const picked = await open({
      multiple: true,
      title: "Import video into the Clip pool",
      filters: [{ name: "Video", extensions: VIDEO_EXTENSIONS }],
    });
    const paths = picked === null ? [] : Array.isArray(picked) ? picked : [picked];
    setError(undefined);
    for (const path of paths) {
      setImporting(path.split("/").pop() ?? path);
      try {
        await clipApi.importClip(path);
      } catch (raw) {
        setError(toCoreError(raw));
      }
    }
    setImporting(null);
    if (paths.length > 0) setClips(await clipApi.listClips());
  }

  function replace(updated: Clip) {
    setClips((cs) => cs?.map((c) => (c.id === updated.id ? updated : c)) ?? null);
  }

  return (
    <section>
      <div className="clips-header">
        <h1>Clips</h1>
        <button disabled={importing !== null} onClick={importVideo}>
          {importing ? `Importing ${importing}…` : "Import video…"}
        </button>
      </div>
      {error && <p className="error">{error.message}</p>}
      {clips && clips.length === 0 && <p className="muted">No Clips yet. Import video to start the pool.</p>}
      <ul className="clip-grid">
        {clips?.map((c) => (
          <ClipCard key={c.id} clip={c} onChange={replace} onError={setError} />
        ))}
      </ul>
    </section>
  );
}

export const clipsArea: Area = { id: "clips", label: "Clips", Screen: ClipsScreen };
