import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { styleApi } from "./api";
import { styleSections } from "./sections";
import { styleStatusLabel, type Style, type StyleStatus } from "./types";
import "./style.css";

interface Props {
  id: number;
  /** Label of the screen that `onBack` returns to, for example the Thesis title. */
  backLabel: string;
  onBack(): void;
}

/** One Style: its header and status moves, then every registered section. */
export function StyleView({ id, backLabel, onBack }: Props) {
  const [style, setStyle] = useState<Style>();
  const [error, setError] = useState<CoreError>();
  const [confirmCut, setConfirmCut] = useState(false);

  const reload = useCallback(() => {
    styleApi.getStyle(id).then(setStyle, (raw) => setError(toCoreError(raw)));
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

  return (
    <section className="style-view">
      <button className="back" onClick={onBack}>
        ← {backLabel}
      </button>
      {error && <p className="error">{error.message}</p>}
      {style && (
        <>
          <header>
            <p className="status">{styleStatusLabel[style.status]}</p>
            <h1>{style.name}</h1>
            <p className="color">{style.color}</p>
            {style.moves.length > 0 && (
              <div className="actions">
                {style.moves
                  .filter((to) => to !== "cut")
                  .map((to) => (
                    <button key={to} onClick={() => move(to)}>
                      Move to {styleStatusLabel[to]}
                    </button>
                  ))}
                {style.moves.includes("cut") &&
                  (confirmCut ? (
                    <button onClick={() => move("cut")}>Confirm: cut this Style</button>
                  ) : (
                    <button onClick={() => setConfirmCut(true)}>Cut Style</button>
                  ))}
              </div>
            )}
          </header>
          {styleSections.map(({ id: sectionId, title, Component }) => (
            <section key={sectionId} className="style-section">
              <h2>{title}</h2>
              <Component style={style} onStyleChanged={reload} />
            </section>
          ))}
        </>
      )}
    </section>
  );
}
