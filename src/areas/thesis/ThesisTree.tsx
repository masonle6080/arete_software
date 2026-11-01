import { useCallback, useEffect, useState, type ReactNode } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { thesisTabLabel, type View } from "../../shell/sidebar";
import { listenTreeChanges } from "../../shell/treeChanges";
import { itemView, loadThesisContents, type ThesisContents } from "./contents";

interface Props {
  thesisId: number;
  /** The key of the open row (see `openRow`). */
  openKey: string;
  onGo(view: View): void;
}

/** The parts of the tree that open and close. */
type Group = "essays" | "moodboards" | "drop";

/**
 * The source-list tree of the current Thesis: Overview, Essays, Moodboards,
 * Drop with each Style, and Documentary. Essays, Moodboards, and Drop open
 * and close. The tree reloads on every announced tree change.
 */
export function ThesisTree({ thesisId, openKey, onGo }: Props) {
  const [contents, setContents] = useState<ThesisContents>();
  const [error, setError] = useState<CoreError>();
  const [closed, setClosed] = useState<Set<Group>>(() => new Set());

  const reload = useCallback(() => {
    loadThesisContents(thesisId).then(
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

  const tab = (id: string) => row(`thesis:${id}`, thesisTabLabel[id], itemView.tab(id), "tree-top");

  const group = (id: Group, children: ReactNode[]) => {
    const label = thesisTabLabel[id];
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
          <ul className="tree-inline">{tab(id)}</ul>
        </div>
        {open && children.length > 0 && <ul className="tree-children">{children}</ul>}
      </li>
    );
  };

  return (
    <div className="thesis-tree">
      {error && <p className="error">{error.message}</p>}
      <ul>
        {tab("overview")}
        {group(
          "essays",
          (contents?.essays ?? []).map((e) =>
            row(`essay:${e.id}`, e.title || "Untitled", itemView.essay(e), "tree-leaf"),
          ),
        )}
        {group(
          "moodboards",
          (contents?.moodboards ?? []).map((b) =>
            row(`moodboard:${b.id}`, b.title, itemView.moodboard(b), "tree-leaf"),
          ),
        )}
        {group(
          "drop",
          (contents?.styles ?? []).map((s) =>
            row(
              `style:${s.id}`,
              <>
                {s.name} <span className="tree-color">{s.color}</span>
              </>,
              itemView.style(s),
              s.status === "cut" ? "tree-leaf cut" : "tree-leaf",
            ),
          ),
        )}
        {tab("documentary")}
      </ul>
    </div>
  );
}
