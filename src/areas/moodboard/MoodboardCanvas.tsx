import { useCallback, useEffect, useRef, useState } from "react";
import { Excalidraw, getSceneVersion, serializeAsJSON } from "@excalidraw/excalidraw";
import "@excalidraw/excalidraw/index.css";
import type { ExcalidrawInitialDataState, BinaryFileData, ExcalidrawImperativeAPI } from "@excalidraw/excalidraw/types";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { mediaApi, moodboardApi } from "./api";
import { mimeTypeOf } from "./mediaType";
import { PassageLinkPanel } from "./PassageLinkPanel";

interface Props {
  moodboardId: number;
}

type Elements = Parameters<typeof getSceneVersion>[0];
type AppStateArg = Parameters<typeof serializeAsJSON>[1];

const SAVE_DELAY_MS = 500;

/**
 * The canvas of one Moodboard (Excalidraw, ADR-0003). It loads the saved
 * scene, saves changes after a short pause, and imports every dropped or
 * pasted image into the library; the scene keeps only the media name as the
 * image's `fileId`. It knows nothing about the owner, so any owner reuses it.
 * With one item selected, a panel links it to a Passage or unlinks it.
 */
export default function MoodboardCanvas({ moodboardId }: Props) {
  const [initial, setInitial] = useState<ExcalidrawInitialDataState>();
  const [error, setError] = useState<CoreError>();
  const pending = useRef<{ elements: Elements; appState: AppStateArg }>();
  const savedVersion = useRef<number>();
  const timer = useRef<ReturnType<typeof setTimeout>>();
  const [api, setApi] = useState<ExcalidrawImperativeAPI>();
  const [selected, setSelected] = useState<{ id: string; passageId?: string }>();

  useEffect(() => {
    let live = true;
    setInitial(undefined);
    loadScene(moodboardId).then(
      (data) => {
        if (!live) return;
        savedVersion.current = getSceneVersion((data.elements ?? []) as Elements);
        setInitial(data);
      },
      (raw) => live && setError(toCoreError(raw)),
    );
    return () => {
      live = false;
    };
  }, [moodboardId]);

  const flush = useCallback(() => {
    clearTimeout(timer.current);
    const next = pending.current;
    if (!next) return;
    pending.current = undefined;
    const version = getSceneVersion(next.elements);
    if (version === savedVersion.current) return;
    const scene = serializeAsJSON(next.elements, next.appState, {}, "database");
    moodboardApi.saveScene(moodboardId, scene).then(
      () => {
        savedVersion.current = version;
        setError(undefined);
      },
      (raw) => setError(toCoreError(raw)),
    );
  }, [moodboardId]);

  // Save what is pending when the founder leaves this Moodboard.
  useEffect(() => flush, [flush]);

  const onChange = useCallback(
    (elements: Elements, appState: AppStateArg) => {
      pending.current = { elements, appState };
      setSelected((current) => {
        const next = selectedItem(elements, appState);
        return current?.id === next?.id && current?.passageId === next?.passageId ? current : next;
      });
      clearTimeout(timer.current);
      timer.current = setTimeout(flush, SAVE_DELAY_MS);
    },
    [flush],
  );

  const importImage = useCallback(async (file: File) => {
    try {
      return await mediaApi.importFile(file);
    } catch (raw) {
      setError(toCoreError(raw));
      throw raw;
    }
  }, []);

  return (
    <div className="moodboard-canvas">
      {error && <p className="error">{error.message}</p>}
      {api && selected && (
        <PassageLinkPanel api={api} elementId={selected.id} passageId={selected.passageId} />
      )}
      {initial && (
        <Excalidraw
          initialData={initial}
          excalidrawAPI={setApi}
          onChange={onChange}
          generateIdForFile={importImage}
          UIOptions={{ canvasActions: { loadScene: false, saveToActiveFile: false, export: false } }}
        />
      )}
    </div>
  );
}

/** The one selected element and its linked Passage, or undefined. */
function selectedItem(elements: Elements, appState: AppStateArg) {
  const chosen = appState.selectedElementIds ?? {};
  const ids = Object.keys(chosen).filter((id) => chosen[id]);
  if (ids.length !== 1) return undefined;
  const el = elements.find((e) => e.id === ids[0] && !e.isDeleted);
  if (!el) return undefined;
  const passageId = el.customData?.passageId;
  return { id: el.id, passageId: typeof passageId === "string" ? passageId : undefined };
}

/** The saved scene plus the image bytes it references, read from the library. */
async function loadScene(id: number): Promise<ExcalidrawInitialDataState> {
  const raw = await moodboardApi.loadScene(id);
  if (!raw) return { elements: [] };
  const saved = JSON.parse(raw) as ExcalidrawInitialDataState;
  const fileIds = new Set<string>();
  for (const el of saved.elements ?? []) {
    if (el.type === "image" && el.fileId) fileIds.add(el.fileId);
  }
  const files = await Promise.all([...fileIds].map(loadFile));
  const found = files.filter((f): f is BinaryFileData => f !== undefined);
  return { ...saved, files: Object.fromEntries(found.map((f) => [f.id, f])) };
}

/** One image for the canvas, or undefined when its media cannot be read. */
async function loadFile(fileId: string): Promise<BinaryFileData | undefined> {
  try {
    const bytes = await mediaApi.read(fileId);
    const mimeType = mimeTypeOf(fileId);
    return {
      id: fileId as BinaryFileData["id"],
      mimeType: mimeType as BinaryFileData["mimeType"],
      dataURL: (await toDataURL(new Blob([bytes], { type: mimeType }))) as BinaryFileData["dataURL"],
      created: Date.now(),
    };
  } catch {
    return undefined; // Excalidraw shows a placeholder for a missing image.
  }
}

function toDataURL(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}
