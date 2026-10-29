import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { toCoreError } from "../../../shell/coreError";
import { clipApi, type Clip } from "../../clips/api";
import type { PassageSection, PassageSectionProps } from ".";

/** The Clips linked to a Passage. Link and unlink them in the Clips area. */
function LinkedClips({ passage }: PassageSectionProps) {
  const [clips, setClips] = useState<Clip[]>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    clipApi.passageClips(passage.id).then(setClips, (e) => setError(toCoreError(e).message));
  }, [passage.id]);

  if (error) return <p className="error">{error}</p>;
  if (!clips) return null;
  if (clips.length === 0) return <p className="muted">No Clips are linked to this Passage.</p>;
  return (
    <ul className="passage-clips">
      {clips.map((c) => (
        <li key={c.id}>
          {c.thumbnailPath ? (
            <img src={convertFileSrc(c.thumbnailPath)} alt="" />
          ) : (
            <div className="thumb-empty" />
          )}
          <div>
            <div className="name">{c.originalName}</div>
            <div className="muted">
              {c.dateShot ?? "Date unknown"}
              {c.tags.length > 0 && ` · ${c.tags.map((t) => `#${t}`).join(" ")}`}
            </div>
            {c.notes && <div className="muted">{c.notes}</div>}
          </div>
        </li>
      ))}
    </ul>
  );
}

export const clipsSection: PassageSection = { id: "clips", title: "Clips", Component: LinkedClips };
