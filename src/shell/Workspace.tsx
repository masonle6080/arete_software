import { useCallback, useEffect, useRef, useState } from "react";
import { settingsArea, sharedAreas } from "../areas";
import type { LibraryInfo } from "../areas/library/session";
import { loadCurrentThesis, saveCurrentThesis, ThesisList, ThesisPicker, ThesisTree } from "../areas/thesis";
import { thesisApi } from "../areas/thesis/api";
import { Breadcrumb, Button } from "../ui";
import { runLeaveGuards } from "./leaveGuards";
import { crumbs, placeKey, pop, popTo, push, relabel, screenLevel, type NavPath, type Step } from "./navPath";
import { PlaceScreen } from "./PlaceScreen";
import {
  blocksBackShortcut,
  isBackShortcut,
  openRow,
  openStep,
  pathOf,
  shortcutTab,
  type PathNames,
  type View,
} from "./sidebar";

interface Props {
  library: LibraryInfo;
  onSwitchLibrary(): void;
}

const areas = [...sharedAreas, settingsArea];
const areaLabel = (id: string) => areas.find((a) => a.id === id)?.label ?? id;

/**
 * The open library: a source-list sidebar and the content pane. The sidebar
 * picks the current Thesis and shows it as a tree (ADR-0004). Below the tree
 * are the Shared areas, then Settings. The content pane shows the top of the
 * navigation path under its breadcrumb; ⌘[ goes back one level.
 */
export function Workspace({ library, onSwitchLibrary }: Props) {
  const [thesisId, setThesisId] = useState(() => loadCurrentThesis(library.path));
  const [thesisTitle, setThesisTitle] = useState("");
  const names = useRef<PathNames>({ thesis: "", area: areaLabel });
  names.current = { thesis: thesisTitle, area: areaLabel };

  const [path, setPath] = useState<NavPath>(() =>
    pathOf(thesisId === undefined ? { kind: "theses" } : { kind: "thesis", tab: "overview" }, names.current),
  );
  // Each sidebar click remounts the pane, so clicking the open row returns to its start.
  const [visit, setVisit] = useState(0);
  const [thesesVersion, setThesesVersion] = useState(0);

  // The root of a Thesis path is the Thesis title.
  useEffect(() => {
    if (thesisId === undefined) return;
    let live = true;
    thesisApi.getThesis(thesisId).then(
      (t) => live && setThesisTitle(t.title),
      () => undefined, // The Thesis picker reports a missing Thesis.
    );
    return () => {
      live = false;
    };
  }, [thesisId, thesesVersion]);
  useEffect(() => {
    setPath((p) => (p[0].place.kind === "thesis" ? relabel(p, 0, thesisTitle) : p));
  }, [thesisTitle]);

  /** Changes the path once the open screen is ready to leave (the Essay editor saves first). */
  const navigate = useCallback(async (change: (p: NavPath) => NavPath, remount = false) => {
    await runLeaveGuards();
    setPath(change);
    if (remount) setVisit((n) => n + 1);
  }, []);

  const go = useCallback((view: View) => navigate(() => pathOf(view, names.current), true), [navigate]);
  const makeCurrent = useCallback(
    (id: number | undefined) => {
      setThesisId(id);
      saveCurrentThesis(library.path, id);
    },
    [library.path],
  );

  /** Opens an item, for example from a chip. A Thesis tab of another Thesis makes that Thesis current. */
  const open = useCallback(
    async (step: Step) => {
      await runLeaveGuards();
      // Which Thesis becomes current depends on the step only, not on the path.
      const switchTo = openStep(path, step, thesisId, names.current).thesisId;
      if (switchTo !== undefined) makeCurrent(switchTo);
      setPath((p) => openStep(p, step, thesisId, names.current).path);
    },
    [path, thesisId, makeCurrent],
  );

  const pickThesis = useCallback(
    (id: number | undefined) => {
      makeCurrent(id);
      if (id === undefined) return void go({ kind: "theses" });
      const tab = path[0].place.kind === "thesis" && path[1]?.place.kind === "thesis" ? path[1].place.tab : "overview";
      void go({ kind: "thesis", tab });
    },
    [makeCurrent, go, path],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      if (isBackShortcut(e)) {
        // The Moodboard canvas and text fields use ⌘[ themselves.
        if (blocksBackShortcut(e.target as Element | null)) return;
        e.preventDefault();
        void navigate(pop);
        return;
      }
      const tab = thesisId === undefined ? undefined : shortcutTab(e);
      if (tab === undefined) return;
      e.preventDefault();
      void go({ kind: "thesis", tab });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [thesisId, go, navigate]);

  const openKey = openRow(path);
  const navButton = (next: View, label: string) => {
    const isOpen = openRow(pathOf(next, names.current)) === openKey;
    return (
      <Button
        key={label}
        className={isOpen ? "active" : undefined}
        aria-current={isOpen ? "page" : undefined}
        onClick={() => void go(next)}
      >
        {label}
      </Button>
    );
  };

  const { depth, passageId } = screenLevel(path);
  const place = path[depth].place;
  const key = `${visit}:${depth}:${placeKey(place)}`;
  const area = place.kind === "area" ? areas.find((a) => a.id === place.id) : undefined;

  return (
    <div className="shell">
      <nav className="sidebar">
        <div className="drag-region" data-tauri-drag-region />
        <ThesisPicker
          value={thesisId}
          version={thesesVersion}
          onPick={pickThesis}
          onNewThesis={() => void go({ kind: "theses", create: true })}
          onAllTheses={() => void go({ kind: "theses" })}
        />
        {thesisId !== undefined && <ThesisTree thesisId={thesisId} openKey={openKey} onGo={(v) => void go(v)} />}
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
        <div className="drag-region content-titlebar" data-tauri-drag-region>
          <Breadcrumb items={crumbs(path)} onOpen={(d) => void navigate((p) => popTo(p, d))} />
        </div>
        <main className="content">
          {place.kind === "theses" ? (
            <ThesisList
              key={key}
              focusCreate={place.create}
              onOpen={pickThesis}
              onChanged={() => setThesesVersion((n) => n + 1)}
            />
          ) : place.kind === "area" ? (
            area && <area.Screen key={key} library={library} onOpen={open} />
          ) : (
            <PlaceScreen
              key={key}
              place={place}
              passageId={passageId}
              thesisId={thesisId}
              onPush={open}
              onGo={(v) => void go(v)}
              onReplace={(step) => void navigate((p) => push(pop(p), step))}
              onRelabel={(label) =>
                setPath((p) => (p[depth] && placeKey(p[depth].place) === placeKey(place) ? relabel(p, depth, label) : p))
              }
            />
          )}
        </main>
      </div>
    </div>
  );
}
