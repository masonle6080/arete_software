import { useCallback, useEffect, useState, type FormEvent } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { announceTreeChange } from "../../shell/treeChanges";
import { Badge, Button, SectionHeader, SegmentedControl } from "../../ui";
import { dropApi } from "../drop/api";
import { steps, type Step } from "../../shell/navPath";
import { styleApi } from "./api";
import { styleSections } from "./sections";
import { baseKindOf, sectionsIn, styleSegments, type BaseKind, type StyleSegment } from "./segments";
import { StyleChips } from "./StyleChips";
import { StyleTraceExport } from "./StyleTraceExport";
import { styleStatusLabel, type Style, type StyleStatus } from "./types";
import "./style.css";

/**
 * The Base kind switch at the top of Build. A Style uses one kind of base,
 * so Build shows one section; the switch reaches the other kind, for
 * example to move a Tech Pack Style back to a Blank.
 */
const baseKinds: readonly { value: BaseKind; label: string }[] = [
  { value: "blank", label: "Blank" },
  { value: "techPack", label: "Tech Pack" },
];

interface Props {
  id: number;
  /** Opens an item one level below this Style in the navigation path, for example its Moodboard. */
  onOpen(step: Step): void;
  /** Opens another Style in place of this one, for example a copy just made. */
  onOpenStyle(step: Step): void;
}

/**
 * One Style: its header (status moves, Duplicate, Export), then the sections
 * of one segment: Design, Build, or Make.
 */
export function StyleView({ id, onOpen, onOpenStyle }: Props) {
  const [style, setStyle] = useState<Style>();
  const [error, setError] = useState<CoreError>();
  const [confirmCut, setConfirmCut] = useState(false);
  /** The color typed for a copy; `undefined` while the duplicate form is closed. */
  const [copyColor, setCopyColor] = useState<string>();

  /** Whether the Style's Drop takes duplicated Styles now. */
  const [canDuplicate, setCanDuplicate] = useState(false);

  /** The open segment. It stays open when the Style reloads. */
  const [segment, setSegment] = useState<StyleSegment>("design");
  /**
   * The base kind the Build segment shows. It follows the Style's base each
   * time the Style loads; the Base kind switch changes it to start the other
   * kind of base.
   */
  const [baseKind, setBaseKind] = useState<BaseKind>("techPack");

  const reload = useCallback(() => {
    styleApi
      .getStyle(id)
      .then(async (s) => {
        const [drop, base] = await Promise.all([dropApi.getDrop(s.dropId), styleApi.styleBase(s.id)]);
        setStyle(s);
        setCanDuplicate(drop.canAddStyles);
        setBaseKind(baseKindOf(base));
      })
      .catch((raw) => setError(toCoreError(raw)));
  }, [id]);
  useEffect(reload, [reload]);

  async function move(to: StyleStatus) {
    setError(undefined);
    setConfirmCut(false);
    try {
      setStyle(await styleApi.moveStyle(id, to));
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  async function duplicate(e: FormEvent) {
    e.preventDefault();
    setError(undefined);
    try {
      const copy = await styleApi.duplicateStyle(id, copyColor ?? "");
      setCopyColor(undefined);
      announceTreeChange();
      onOpenStyle(steps.style(copy));
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <section className="style-view">
      {error && <p className="error">{error.message}</p>}
      {style && (
        <>
          <header>
            {style.cover && (
              <img className="style-cover" src={convertFileSrc(style.cover.path)} alt={`Cover of ${style.name}`} />
            )}
            <Badge lifecycle="style" status={style.status} />
            <h1>{style.name}</h1>
            <p className="color">{style.color}</p>
            <StyleChips style={style} onOpen={onOpen} onSegment={setSegment} />
            {style.moves.length > 0 && (
              <div className="actions">
                {style.moves
                  .filter((to) => to !== "cut")
                  .map((to) => (
                    <Button key={to} onClick={() => move(to)}>
                      Move to {styleStatusLabel[to]}
                    </Button>
                  ))}
                {style.moves.includes("cut") &&
                  (confirmCut ? (
                    <Button variant="destructive" onClick={() => move("cut")}>
                      Confirm: cut this Style
                    </Button>
                  ) : (
                    <Button variant="destructive" onClick={() => setConfirmCut(true)}>
                      Cut Style
                    </Button>
                  ))}
              </div>
            )}
            {!canDuplicate ? null : copyColor === undefined ? (
              <div className="actions">
                <Button onClick={() => setCopyColor("")}>Duplicate in another color</Button>
              </div>
            ) : (
              <form className="actions" onSubmit={duplicate}>
                <input
                  autoFocus
                  aria-label="Color of the copy"
                  placeholder="Color, for example Black"
                  value={copyColor}
                  onChange={(e) => setCopyColor(e.target.value)}
                />
                <Button type="submit" variant="primary" disabled={!copyColor.trim()}>
                  Duplicate
                </Button>
                <Button type="button" onClick={() => setCopyColor(undefined)}>
                  Cancel
                </Button>
              </form>
            )}
            <StyleTraceExport style={style} />
            <SegmentedControl<StyleSegment> label="Style segment" options={styleSegments} value={segment} onChange={setSegment} />
          </header>
          {segment === "build" && (
            <div className="style-base-kind">
              <SegmentedControl<BaseKind> label="Base kind" options={baseKinds} value={baseKind} onChange={setBaseKind} />
            </div>
          )}
          {sectionsIn(styleSections, segment, baseKind).map(({ id: sectionId, title, Component }) => (
            <section key={sectionId} className="style-section">
              <SectionHeader title={title} />
              <Component style={style} onStyleChanged={reload} onOpen={onOpen} />
            </section>
          ))}
        </>
      )}
    </section>
  );
}
