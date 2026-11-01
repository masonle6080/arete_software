import { useCallback, useState } from "react";
import { libraryAreas, settingsArea } from "../areas";
import type { LibraryInfo } from "../areas/library/session";
import {
  loadCurrentThesis,
  saveCurrentThesis,
  ThesisList,
  ThesisPicker,
  thesisTabs,
  ThesisWorkspace,
} from "../areas/thesis";
import { Button } from "../ui";

/** What the content pane shows: a tab of the current Thesis, every Thesis, or one area. */
type View = { kind: "thesis"; tab: string } | { kind: "theses" } | { kind: "area"; id: string };

/** One text per View, to mark the active sidebar button. */
const viewKey = (v: View) => (v.kind === "thesis" ? `thesis:${v.tab}` : v.kind === "area" ? `area:${v.id}` : v.kind);

interface Props {
  library: LibraryInfo;
  onSwitchLibrary(): void;
}

/**
 * The open library: a sidebar centered on the current Thesis, and the content
 * pane. The sidebar picks the Thesis and opens its tabs. Below them are every
 * Thesis, the library-wide areas (the Library group), and Settings.
 */
export function Workspace({ library, onSwitchLibrary }: Props) {
  const [thesisId, setThesisId] = useState(() => loadCurrentThesis(library.path));
  const [view, setView] = useState<View>(() =>
    thesisId === undefined ? { kind: "theses" } : { kind: "thesis", tab: "overview" },
  );
  // Each sidebar click remounts the pane, so clicking the open tab returns to its start.
  const [visit, setVisit] = useState(0);
  const [thesesVersion, setThesesVersion] = useState(0);
  const [libraryOpen, setLibraryOpen] = useState(false);

  const go = useCallback((next: View) => {
    setView(next);
    setVisit((n) => n + 1);
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

  const area = view.kind === "area" ? [...libraryAreas, settingsArea].find((a) => a.id === view.id) : undefined;
  const navButton = (next: View, label: string, className = "") => (
    <Button
      key={label}
      className={viewKey(next) === viewKey(view) ? `${className} active` : className}
      onClick={() => go(next)}
    >
      {label}
    </Button>
  );

  return (
    <div className="shell">
      <nav className="sidebar">
        <div className="drag-region" data-tauri-drag-region />
        <ThesisPicker value={thesisId} version={thesesVersion} onPick={pickThesis} />
        {thesisId !== undefined && (
          <div className="sidebar-group">
            {thesisTabs.map((t) => navButton({ kind: "thesis", tab: t.id }, t.title))}
          </div>
        )}
        <div className="sidebar-group">
          {navButton({ kind: "theses" }, "All Theses")}
          <Button className="group-toggle" aria-expanded={libraryOpen} onClick={() => setLibraryOpen((o) => !o)}>
            Library {libraryOpen ? "▾" : "▸"}
          </Button>
          {libraryOpen && libraryAreas.map((a) => navButton({ kind: "area", id: a.id }, a.label, "nested"))}
          {navButton({ kind: "area", id: settingsArea.id }, settingsArea.label)}
        </div>
        <Button className="switch" onClick={onSwitchLibrary}>
          Switch library…
        </Button>
      </nav>
      <div className="content-pane">
        <div className="drag-region" data-tauri-drag-region />
        <main className="content">
          {view.kind === "thesis" && thesisId !== undefined ? (
            <ThesisWorkspace key={`${thesisId}:${visit}`} thesisId={thesisId} tab={view.tab} />
          ) : view.kind === "area" && area ? (
            <area.Screen key={visit} library={library} />
          ) : (
            <ThesisList key={visit} onOpen={pickThesis} onChanged={() => setThesesVersion((n) => n + 1)} />
          )}
        </main>
      </div>
    </div>
  );
}
