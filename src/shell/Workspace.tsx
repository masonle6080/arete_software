import { useCallback, useEffect, useRef, useState } from "react";
import { settingsArea, sharedAreas } from "../areas";
import type { LibraryInfo } from "../areas/library/session";
import { loadCurrentThesis, saveCurrentThesis, ThesisList, ThesisPicker } from "../areas/thesis";
import { thesisApi } from "../areas/thesis/api";
import { Breadcrumb, Button, icons } from "../ui";
import type { Area } from "./area";
import { runLeaveGuards } from "./leaveGuards";
import {
  crumbs,
  placeKey,
  pop,
  popTo,
  push,
  relabel,
  screenLevel,
  thesisTabLabel,
  thesisTabs,
  type NavPath,
  type Step,
  type ThesisTab,
} from "./navPath";
import { PlaceScreen } from "./PlaceScreen";
import { SidebarRows, type SidebarRow } from "./SidebarRows";
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
 * picks the current Thesis or a Shared area from one dropdown. Below it are
 * flat rows: the six tabs of the Thesis, or the Shared areas when one is
 * open (ADR-0004). Settings and "Switch library…" stay at the bottom. The content pane shows the top of the
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
  const areaRow = (a: Area): SidebarRow => ({
    key: `area:${a.id}`,
    label: a.label,
    icon: a.icon,
    onOpen: () => void go({ kind: "area", id: a.id }),
  });
  const tabRow = (tab: ThesisTab): SidebarRow => ({
    key: `thesis:${tab}`,
    label: thesisTabLabel[tab],
    icon: icons[tab],
    onOpen: () => void go({ kind: "thesis", tab }),
  });

  const { depth, passageId } = screenLevel(path);
  const place = path[depth].place;
  const key = `${visit}:${depth}:${placeKey(place)}`;
  const area = place.kind === "area" ? areas.find((a) => a.id === place.id) : undefined;
  const root = path[0].place;
  const openShared = root.kind === "area" && sharedAreas.some((a) => a.id === root.id) ? root.id : undefined;

  return (
    <div className="shell">
      <nav className="sidebar">
        <div className="drag-region" data-tauri-drag-region />
        <ThesisPicker
          value={thesisId}
          version={thesesVersion}
          sharedAreas={sharedAreas}
          openArea={openShared}
          onArea={(id) => void go({ kind: "area", id })}
          onPick={pickThesis}
          onNewThesis={() => void go({ kind: "theses", create: true })}
          onAllTheses={() => void go({ kind: "theses" })}
        />
        {openShared !== undefined ? (
          <SidebarRows rows={sharedAreas.map(areaRow)} openKey={openKey} />
        ) : (
          thesisId !== undefined && <SidebarRows rows={thesisTabs.map(tabRow)} openKey={openKey} />
        )}
        <SidebarRows rows={[areaRow(settingsArea)]} openKey={openKey} />
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
