import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { announceTreeChange } from "../../shell/treeChanges";
import { Badge, Button, SectionHeader } from "../../ui";
import { dropApi } from "../drop/api";
import type { Moodboard } from "../moodboard/types";
import { styleApi } from "./api";
import { styleSections } from "./sections";
import { styleStatusLabel, type Style, type StyleStatus } from "./types";
import "./style.css";

interface Props {
  id: number;
  /** Label of the screen that `onBack` returns to, for example the Thesis title. */
  backLabel: string;
  onBack(): void;
  /** Opens another Style, for example a copy just made. */
  onOpenStyle?(id: number): void;
  /** Opens the Style's Moodboard on its own screen. */
  onOpenMoodboard(board: Moodboard): void;
}

/** One Style: its header and status moves, then every registered section. */
export function StyleView({ id, backLabel, onBack, onOpenStyle, onOpenMoodboard }: Props) {
  const [style, setStyle] = useState<Style>();
  const [error, setError] = useState<CoreError>();
  const [confirmCut, setConfirmCut] = useState(false);
  /** The color typed for a copy; `undefined` while the duplicate form is closed. */
  const [copyColor, setCopyColor] = useState<string>();

  /** Whether the Style's Drop takes duplicated Styles now. */
  const [canDuplicate, setCanDuplicate] = useState(false);

  const reload = useCallback(() => {
    styleApi
      .getStyle(id)
      .then(async (s) => {
        setStyle(s);
        setCanDuplicate((await dropApi.getDrop(s.dropId)).canAddStyles);
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
      onOpenStyle?.(copy.id);
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <section className="style-view">
      <Button variant="plain" className="back" onClick={onBack}>
        ← {backLabel}
      </Button>
      {error && <p className="error">{error.message}</p>}
      {style && (
        <>
          <header>
            <Badge lifecycle="style" status={style.status} />
            <h1>{style.name}</h1>
            <p className="color">{style.color}</p>
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
          </header>
          {styleSections.map(({ id: sectionId, title, Component }) => (
            <section key={sectionId} className="style-section">
              <SectionHeader title={title} />
              <Component style={style} onStyleChanged={reload} onOpenMoodboard={onOpenMoodboard} />
            </section>
          ))}
        </>
      )}
    </section>
  );
}
