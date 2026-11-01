import { useCallback, useEffect, useState, type ReactNode } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import type { View } from "../../shell/sidebar";
import { listenTreeChanges } from "../../shell/treeChanges";
import { dropApi } from "../drop/api";
import { essayThesisApi, type DocumentSummary } from "../essays/api";
import { moodboardApi } from "../moodboard/api";
import type { Moodboard } from "../moodboard/types";
import { styleApi } from "../style/api";
import type { Style } from "../style/types";

interface Props {
  thesisId: number;
  /** The key of the open row (see `openRow`). */
  openKey: string;
  onGo(view: View): void;
}

/** The parts of the tree that open and close. */
type Group = "essays" | "moodboards" | "drop";

interface Contents {
  essays: DocumentSummary[];
  moodboards: Moodboard[];
  /** The Styles of the Thesis's active (not cancelled) Drop. Empty when it has none. */
  styles: Style[];
}

async function loadContents(thesisId: number): Promise<Contents> {
  const [essays, moodboards, drops] = await Promise.all([
    essayThesisApi.essaysOfThesis(thesisId),
    moodboardApi.listMoodboards({ kind: "thesis", id: thesisId }),
    dropApi.thesisDrops(thesisId),
  ]);
  const active = drops.drops.find((d) => d.stage !== "cancelled");
  const styles = active ? await styleApi.dropStyles(active.id) : [];
  return { essays, moodboards, styles };
}

/**
 * The source-list tree of the current Thesis: Overview, Essays, Moodboards,
 * Drop with each Style, and Documentary. Essays, Moodboards, and Drop open
 * and close. The tree reloads on every announced tree change.
 */
export function ThesisTree({ thesisId, openKey, onGo }: Props) {
  const [contents, setContents] = useState<Contents>();
  const [error, setError] = useState<CoreError>();
  const [closed, setClosed] = useState<Set<Group>>(() => new Set());

  const reload = useCallback(() => {
    loadContents(thesisId).then(
      (c) => {
        setContents(c);
        setError(undefined);
      },
      (raw) => setError(toCoreError(raw)),
    );
  }, [thesisId]);
  useEffect(() => {
    reload();
    return listenTreeChanges(reload);
  }, [reload]);

  const toggle = (group: Group) =>
    setClosed((s) => {
      const next = new Set(s);
      if (!next.delete(group)) next.add(group);
      return next;
    });

  const row = (key: string, label: ReactNode, view: View, className: string) => (
    <li key={key}>
      <button
        type="button"
        className={`tree-row ${className}${key === openKey ? " active" : ""}`}
        aria-current={key === openKey ? "page" : undefined}
        onClick={() => onGo(view)}
      >
        {label}
      </button>
    </li>
  );

  const tab = (id: string, label: string) => row(`thesis:${id}`, label, { kind: "thesis", tab: id }, "tree-top");

  const group = (id: Group, label: string, children: ReactNode[]) => {
    const open = !closed.has(id);
    return (
      <li key={id}>
        <div className="tree-group">
          <button
            type="button"
            className="tree-disclosure"
            aria-expanded={open}
            aria-label={`${open ? "Close" : "Open"} ${label}`}
            onClick={() => toggle(id)}
          >
            {open ? "▾" : "▸"}
          </button>
          <ul className="tree-inline">{tab(id, label)}</ul>
        </div>
        {open && children.length > 0 && <ul className="tree-children">{children}</ul>}
      </li>
    );
  };

  return (
    <div className="thesis-tree">
      {error && <p className="error">{error.message}</p>}
      <ul>
        {tab("overview", "Overview")}
        {group(
          "essays",
          "Essays",
          (contents?.essays ?? []).map((e) =>
            row(`essay:${e.id}`, e.title || "Untitled", { kind: "thesis", tab: "essays", open: { kind: "essay", id: e.id } }, "tree-leaf"),
          ),
        )}
        {group(
          "moodboards",
          "Moodboards",
          (contents?.moodboards ?? []).map((b) =>
            row(`moodboard:${b.id}`, b.title, { kind: "thesis", tab: "moodboards", open: { kind: "moodboard", board: b } }, "tree-leaf"),
          ),
        )}
        {group(
          "drop",
          "Drop",
          (contents?.styles ?? []).map((s) =>
            row(
              `style:${s.id}`,
              <>
                {s.name} <span className="tree-color">{s.color}</span>
              </>,
              { kind: "thesis", tab: "drop", open: { kind: "style", id: s.id } },
              s.status === "cut" ? "tree-leaf cut" : "tree-leaf",
            ),
          ),
        )}
        {tab("documentary", "Documentary")}
      </ul>
    </div>
  );
}
