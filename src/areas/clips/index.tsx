import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Area } from "../../shell/area";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { documentaryApi } from "../documentary/api";
import type { Documentary } from "../documentary/types";
import { clipApi, type Clip, type ClipFilter } from "./api";
import { ClipDetails, passageLabel, useAllPassages } from "./ClipDetails";
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

interface ClipDocumentariesProps {
  clip: Clip;
  all: Documentary[];
  memberOf: Set<number>;
  onChanged(): void;
  onError(e: CoreError): void;
}

/** The Documentaries a Clip is in, with controls to add and remove it. */
function ClipDocumentaries({ clip, all, memberOf, onChanged, onError }: ClipDocumentariesProps) {
  async function run(action: () => Promise<unknown>) {
    try {
      await action();
    } catch (raw) {
      onError(toCoreError(raw));
    }
    onChanged();
  }

  if (all.length === 0) return null;
  const inDocs = all.filter((d) => memberOf.has(d.id));
  const others = all.filter((d) => !memberOf.has(d.id));
  return (
    <div className="documentaries">
      {inDocs.map((d) => (
        <span key={d.id} className="tag">
          {d.title}
          <button title={`Remove from ${d.title}`} onClick={() => run(() => documentaryApi.removeClip(d.id, clip.id))}>
            ×
          </button>
        </span>
      ))}
      {others.length > 0 && (
        <select
          value=""
          onChange={(e) => {
            const id = Number(e.target.value);
            if (id) run(() => documentaryApi.addClip(id, clip.id));
          }}
        >
          <option value="">Add to Documentary…</option>
          {others.map((d) => (
            <option key={d.id} value={d.id}>
              {d.title}
            </option>
          ))}
        </select>
      )}
    </div>
  );
}

function ClipCard({
  clip,
  onChange,
  onError,
  children,
}: {
  clip: Clip;
  onChange(c: Clip): void;
  onError(e: CoreError): void;
  children?: ReactNode;
}) {
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
      {children}
      <button onClick={reveal}>Show in Finder</button>
    </li>
  );
}

function ClipsScreen() {
  const [clips, setClips] = useState<Clip[] | null>(null);
  const [error, setError] = useState<CoreError>();
  const [importing, setImporting] = useState<string | null>(null);
  const [documentaries, setDocumentaries] = useState<Documentary[]>([]);
  // Documentary id → the Clips in it, in pool order.
  const [docClips, setDocClips] = useState<Map<number, Clip[]>>(new Map());
  // The Documentary the pool is filtered by, or null for every Clip.
  const [filter, setFilter] = useState<number | null>(null);
  // The tag and linked-Passage filters, applied by the core.
  const [linkFilter, setLinkFilter] = useState<ClipFilter>({ tag: null, passageId: null });
  const [tags, setTags] = useState<string[]>([]);
  const passageGroups = useAllPassages(setError);

  const reload = useCallback(async () => {
    try {
      const filtered = linkFilter.tag !== null || linkFilter.passageId !== null;
      setClips(await (filtered ? clipApi.filterClips(linkFilter) : clipApi.listClips()));
      setTags(await clipApi.allTags());
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }, [linkFilter]);
  useEffect(() => {
    reload();
  }, [reload]);

  const loadDocumentaries = useCallback(async () => {
    try {
      const docs = await documentaryApi.listDocumentaries();
      const lists = await Promise.all(docs.map((d) => documentaryApi.documentaryClips(d.id)));
      setDocumentaries(docs);
      setDocClips(new Map(docs.map((d, i) => [d.id, lists[i]])));
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }, []);
  useEffect(() => {
    loadDocumentaries();
  }, [loadDocumentaries]);

  const memberships = useMemo(() => {
    const byClip = new Map<number, Set<number>>();
    for (const [docId, list] of docClips) {
      for (const c of list) {
        if (!byClip.has(c.id)) byClip.set(c.id, new Set());
        byClip.get(c.id)!.add(docId);
      }
    }
    return byClip;
  }, [docClips]);

  const shown = filter === null ? clips : (clips?.filter((c) => memberships.get(c.id)?.has(filter)) ?? null);

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
    if (paths.length > 0) await reload();
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
      {documentaries.length > 0 && (
        <label className="clips-filter">
          Documentary
          <select
            value={filter ?? ""}
            onChange={(e) => setFilter(e.target.value === "" ? null : Number(e.target.value))}
          >
            <option value="">All Clips</option>
            {documentaries.map((d) => (
              <option key={d.id} value={d.id}>
                {d.title}
              </option>
            ))}
          </select>
        </label>
      )}
      {tags.length > 0 && (
        <label className="clips-filter">
          Tag
          <select
            value={linkFilter.tag ?? ""}
            onChange={(e) => setLinkFilter((f) => ({ ...f, tag: e.target.value === "" ? null : e.target.value }))}
          >
            <option value="">Any tag</option>
            {tags.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </label>
      )}
      {passageGroups.length > 0 && (
        <label className="clips-filter">
          Passage
          <select
            value={linkFilter.passageId ?? ""}
            onChange={(e) =>
              setLinkFilter((f) => ({ ...f, passageId: e.target.value === "" ? null : e.target.value }))
            }
          >
            <option value="">Any Passage</option>
            {passageGroups.map((g, i) => (
              <optgroup key={i} label={g.title}>
                {g.passages.map((p) => (
                  <option key={p.id} value={p.id}>
                    {passageLabel(p)}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </label>
      )}
      {clips && clips.length === 0 && linkFilter.tag === null && linkFilter.passageId === null && (
        <p className="muted">No Clips yet. Import video to start the pool.</p>
      )}
      {clips && shown?.length === 0 && (filter !== null || linkFilter.tag !== null || linkFilter.passageId !== null) && (
        <p className="muted">No Clips match these filters.</p>
      )}
      <ul className="clip-grid">
        {shown?.map((c) => (
          <ClipCard key={c.id} clip={c} onChange={replace} onError={setError}>
            <ClipDetails
              clip={c}
              passageGroups={passageGroups}
              onChanged={(updated) => {
                if (updated) replace(updated);
                reload();
              }}
              onError={setError}
            />
            <ClipDocumentaries
              clip={c}
              all={documentaries}
              memberOf={memberships.get(c.id) ?? new Set()}
              onChanged={loadDocumentaries}
              onError={setError}
            />
          </ClipCard>
        ))}
      </ul>
    </section>
  );
}

export const clipsArea: Area = { id: "clips", label: "Clips", Screen: ClipsScreen };
