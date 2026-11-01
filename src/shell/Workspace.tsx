import { useCallback, useEffect, useState } from "react";
import { settingsArea, sharedAreas } from "../areas";
import type { LibraryInfo } from "../areas/library/session";
import {
  loadCurrentThesis,
  saveCurrentThesis,
  ThesisList,
  ThesisPicker,
  ThesisTree,
  thesisTabs,
  ThesisWorkspace,
} from "../areas/thesis";
import { Button } from "../ui";
import { openRow, shortcutTab, type ThesisItem, type View } from "./sidebar";

interface Props {
  library: LibraryInfo;
  onSwitchLibrary(): void;
}

const tabIds = thesisTabs.map((t) => t.id);

/**
 * The open library: a source-list sidebar and the content pane. The sidebar
 * picks the current Thesis and shows it as a tree (ADR-0004). Below the tree
 * are the Shared areas, then Settings.
 */
export function Workspace({ library, onSwitchLibrary }: Props) {
  const [thesisId, setThesisId] = useState(() => loadCurrentThesis(library.path));
  const [view, setView] = useState<View>(() =>
    thesisId === undefined ? { kind: "theses" } : { kind: "thesis", tab: "overview" },
  );
  // Each sidebar click remounts the pane, so clicking the open row returns to its start.
  const [visit, setVisit] = useState(0);
  const [thesesVersion, setThesesVersion] = useState(0);

  const go = useCallback((next: View) => {
    setView(next);
    setVisit((n) => n + 1);
  }, []);

  /** Opens an item over the current tab from inside the pane, without a remount. */
  const openItem = useCallback((item: ThesisItem | undefined) => {
    setView((v) => (v.kind === "thesis" ? { kind: "thesis", tab: v.tab, open: item } : v));
  }, []);

  const pickThesis = useCallback(
    (id: number | undefined) => {
      setThesisId(id);
      saveCurrentThesis(library.path, id);
      if (id === undefined) go({ kind: "theses" });
      else go({ kind: "thesis", tab: view.kind === "thesis" ? view.tab : "overview" });
    },
    [library.path, go, view],
  );

  useEffect(() => {
    if (thesisId === undefined) return;
    const onKey = (e: KeyboardEvent) => {
      const tab = shortcutTab(e, tabIds);
      if (tab === undefined) return;
      e.preventDefault();
      go({ kind: "thesis", tab });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [thesisId, go]);

  const openKey = openRow(view);
  const area = view.kind === "area" ? [...sharedAreas, settingsArea].find((a) => a.id === view.id) : undefined;
  const navButton = (next: View, label: string) => (
    <Button
      key={label}
      className={openRow(next) === openKey ? "active" : undefined}
      aria-current={openRow(next) === openKey ? "page" : undefined}
      onClick={() => go(next)}
    >
      {label}
    </Button>
  );

  return (
    <div className="shell">
      <nav className="sidebar">
        <div className="drag-region" data-tauri-drag-region />
        <ThesisPicker
          value={thesisId}
          version={thesesVersion}
          onPick={pickThesis}
          onNewThesis={() => go({ kind: "theses", create: true })}
          onAllTheses={() => go({ kind: "theses" })}
        />
        {thesisId !== undefined && <ThesisTree thesisId={thesisId} openKey={openKey} onGo={go} />}
        <div className="sidebar-group">
          <h2 className="sidebar-heading">Shared</h2>
          {sharedAreas.map((a) => navButton({ kind: "area", id: a.id }, a.label))}
        </div>
        <div className="sidebar-group">{navButton({ kind: "area", id: settingsArea.id }, settingsArea.label)}</div>
        <Button className="switch" onClick={onSwitchLibrary}>
          Switch library…
        </Button>
      </nav>
      <div className="content-pane">
        <div className="drag-region" data-tauri-drag-region />
        <main className="content">
          {view.kind === "thesis" && thesisId !== undefined ? (
            <ThesisWorkspace
              key={`${thesisId}:${visit}`}
              thesisId={thesisId}
              tab={view.tab}
              open={view.open}
              onOpen={openItem}
            />
          ) : view.kind === "area" && area ? (
            <area.Screen key={visit} library={library} />
          ) : (
            <ThesisList
              key={visit}
              focusCreate={view.kind === "theses" && view.create}
              onOpen={pickThesis}
              onChanged={() => setThesesVersion((n) => n + 1)}
            />
          )}
        </main>
      </div>
    </div>
  );
}
