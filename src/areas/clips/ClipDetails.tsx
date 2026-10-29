import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { documentApi } from "../essays/api";
import { passageApi, type Passage } from "../passage/api";
import { clipApi, type Clip } from "./api";

/** Short text of a Passage for a chip or a menu. */
export function passageLabel(p: Passage): string {
  const text = p.lastKnownText.length > 40 ? `${p.lastKnownText.slice(0, 40)}…` : p.lastKnownText;
  return `“${text}”${p.orphaned ? " (orphaned)" : ""}`;
}

/**
 * Every Passage in the library (Essays and Scripts), grouped by document,
 * for the Passage picker and the Passage filter.
 */
export function useAllPassages(onError: (e: CoreError) => void) {
  const [groups, setGroups] = useState<{ title: string; passages: Passage[] }[]>([]);
  useEffect(() => {
    (async () => {
      const docs = [...(await documentApi.list("essay")), ...(await documentApi.list("script"))];
      const lists = await Promise.all(docs.map((d) => passageApi.ofDocument(d.id)));
      setGroups(
        docs
          .map((d, i) => ({ title: d.title || "Untitled", passages: lists[i] }))
          .filter((g) => g.passages.length > 0),
      );
    })().catch((raw) => onError(toCoreError(raw)));
  }, [onError]);
  return groups;
}

interface Props {
  clip: Clip;
  passageGroups: { title: string; passages: Passage[] }[];
  /** Called after any change, with the updated Clip when there is one. */
  onChanged(updated?: Clip): void;
  onError(e: CoreError): void;
}

/** A Clip's tags, notes, and Passage links, with controls to edit them. */
export function ClipDetails({ clip, passageGroups, onChanged, onError }: Props) {
  const [newTag, setNewTag] = useState("");
  const [notes, setNotes] = useState(clip.notes);
  const [linked, setLinked] = useState<Passage[]>([]);

  useEffect(() => setNotes(clip.notes), [clip.notes]);

  const loadLinked = useCallback(() => {
    clipApi.clipPassages(clip.id).then(setLinked, (raw) => onError(toCoreError(raw)));
  }, [clip.id, onError]);
  useEffect(loadLinked, [loadLinked]);

  async function run(action: () => Promise<Clip | void>) {
    try {
      const updated = await action();
      onChanged(updated ?? undefined);
    } catch (raw) {
      onError(toCoreError(raw));
    }
  }

  async function addTag() {
    if (newTag.trim() === "") return;
    await run(() => clipApi.addTag(clip.id, newTag));
    setNewTag("");
  }

  async function saveNotes() {
    if (notes !== clip.notes) await run(() => clipApi.setNotes(clip.id, notes));
  }

  async function link(passageId: string) {
    await run(() => clipApi.linkPassage(clip.id, passageId));
    loadLinked();
  }

  async function unlink(passageId: string) {
    await run(() => clipApi.unlinkPassage(clip.id, passageId));
    loadLinked();
  }

  const linkedIds = new Set(linked.map((p) => p.id));

  return (
    <div className="clip-details">
      <div className="chips">
        {clip.tags.map((t) => (
          <span key={t} className="tag">
            #{t}
            <button title={`Remove tag ${t}`} onClick={() => run(() => clipApi.removeTag(clip.id, t))}>
              ×
            </button>
          </span>
        ))}
        <input
          className="new-tag"
          placeholder="Add tag"
          value={newTag}
          onChange={(e) => setNewTag(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") addTag();
          }}
        />
      </div>
      <textarea
        className="notes"
        placeholder="Notes"
        rows={2}
        value={notes}
        onChange={(e) => setNotes(e.target.value)}
        onBlur={saveNotes}
      />
      <div className="chips">
        {linked.map((p) => (
          <span key={p.id} className="tag" title={p.lastKnownText}>
            {passageLabel(p)}
            <button title="Unlink Passage" onClick={() => unlink(p.id)}>
              ×
            </button>
          </span>
        ))}
        {passageGroups.length > 0 && (
          <select
            value=""
            onChange={(e) => {
              if (e.target.value) link(e.target.value);
            }}
          >
            <option value="">Link to Passage…</option>
            {passageGroups.map((g, i) => (
              <optgroup key={i} label={g.title}>
                {g.passages
                  .filter((p) => !linkedIds.has(p.id))
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {passageLabel(p)}
                    </option>
                  ))}
              </optgroup>
            ))}
          </select>
        )}
      </div>
    </div>
  );
}
