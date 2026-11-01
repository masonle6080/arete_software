import { useEffect, useState, type ReactNode } from "react";
import { useCoreQuery } from "../../shell/coreQuery";
import { thesisTabLabel, thesisTabs, type ThesisTab, type View } from "../../shell/sidebar";
import { listenTreeChanges } from "../../shell/treeChanges";
import { itemView, loadThesisContents } from "./contents";

interface Props {
  thesisId: number;
  /** The key of the open row (see `openRow`). */
  openKey: string;
  onGo(view: View): void;
}

/** The parts of the tree that open and close. */
type Group = Extract<ThesisTab, "essays" | "moodboards" | "drop">;

const isGroup = (tab: ThesisTab): tab is Group => tab === "essays" || tab === "moodboards" || tab === "drop";

/**
 * The source-list tree of the current Thesis: its tabs in `thesisTabs`
 * order (Overview, Essays, Moodboards, Drop with each Style, Documentary). Essays, Moodboards, and Drop open
 * and close. The tree reloads on every announced tree change.
 */
export function ThesisTree({ thesisId, openKey, onGo }: Props) {
  const { data: contents, error, reload } = useCoreQuery(() => loadThesisContents(thesisId), [thesisId]);
  useEffect(() => listenTreeChanges(reload), [reload]);
  const [closed, setClosed] = useState<Set<Group>>(() => new Set());

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

  const tab = (id: ThesisTab) => row(`thesis:${id}`, thesisTabLabel[id], itemView.tab(id), "tree-top");

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

  /** The rows under each group: the Essays, the Moodboards, and the Styles of the active Drop. */
  const children: Record<Group, () => ReactNode[]> = {
    essays: () =>
      (contents?.essays ?? []).map((e) => row(`essay:${e.id}`, e.title || "Untitled", itemView.essay(e), "tree-leaf")),
    moodboards: () =>
      (contents?.moodboards ?? []).map((b) => row(`moodboard:${b.id}`, b.title, itemView.moodboard(b), "tree-leaf")),
    drop: () =>
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
  };

  return (
    <div className="thesis-tree">
      {error && <p className="error">{error.message}</p>}
      <ul>
        {thesisTabs.map((t) => (isGroup(t) ? group(t, children[t]()) : tab(t)))}
      </ul>
    </div>
  );
}
