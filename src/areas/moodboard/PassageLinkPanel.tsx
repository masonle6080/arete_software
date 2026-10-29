import { useEffect, useState } from "react";
import { CaptureUpdateAction, newElementWith } from "@excalidraw/excalidraw";
import type { ExcalidrawImperativeAPI } from "@excalidraw/excalidraw/types";
import { documentApi } from "../essays/api";
import { passageApi, type Passage } from "../passage/api";

interface Props {
  api: ExcalidrawImperativeAPI;
  /** The one selected canvas element. */
  elementId: string;
  /** Its linked Passage ID (`customData.passageId`), if any. */
  passageId: string | undefined;
}

/**
 * Links the selected canvas item to a Passage, or unlinks it. The link lives
 * on the element (`customData.passageId`); the canvas then saves the scene,
 * and the core updates the links it can query.
 */
export function PassageLinkPanel({ api, elementId, passageId }: Props) {
  const passages = usePassages();
  const [choice, setChoice] = useState("");

  function setLink(next: string | undefined) {
    const elements = api.getSceneElements().map((el) => {
      if (el.id !== elementId) return el;
      const { passageId: _old, ...rest } = el.customData ?? {};
      return newElementWith(el, { customData: next ? { ...rest, passageId: next } : rest });
    });
    api.updateScene({ elements, captureUpdate: CaptureUpdateAction.IMMEDIATELY });
    setChoice("");
  }

  if (passageId) {
    const linked = passages?.find((p) => p.id === passageId);
    return (
      <div className="passage-link-panel">
        <span>
          Linked to Passage “{linked ? clip(linked.lastKnownText) : "…"}”{linked?.orphaned && " (orphaned)"}
        </span>
        <button onClick={() => setLink(undefined)}>Unlink</button>
      </div>
    );
  }

  const live = passages?.filter((p) => !p.orphaned) ?? [];
  return (
    <div className="passage-link-panel">
      <select value={choice} onChange={(e) => setChoice(e.target.value)} disabled={live.length === 0}>
        <option value="">{live.length === 0 ? "No Passages to link" : "Link to a Passage…"}</option>
        {live.map((p) => (
          <option key={p.id} value={p.id}>
            {p.documentTitle || "Untitled"}: {clip(p.lastKnownText)}
          </option>
        ))}
      </select>
      <button disabled={!choice} onClick={() => setLink(choice)}>
        Link
      </button>
    </div>
  );
}

function clip(text: string) {
  return text.length > 60 ? `${text.slice(0, 60)}…` : text;
}

/** Every Passage of every Essay and Script, orphaned ones included. */
function usePassages(): Passage[] | undefined {
  const [passages, setPassages] = useState<Passage[]>();
  useEffect(() => {
    let live = true;
    (async () => {
      const docs = [...(await documentApi.list("essay")), ...(await documentApi.list("script"))];
      const lists = await Promise.all(docs.map((d) => passageApi.ofDocument(d.id)));
      if (live) setPassages(lists.flat());
    })().catch(() => live && setPassages([]));
    return () => {
      live = false;
    };
  }, []);
  return passages;
}
