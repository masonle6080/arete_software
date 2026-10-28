import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import type { Drop } from "../drop/types";
import { styleApi } from "./api";
import { styleStatusLabel, type Style } from "./types";
import "./style.css";

interface Props {
  drop: Drop;
  /** Opens the Style view. */
  onOpen(styleId: number): void;
}

/** The Styles of one Drop, Cut ones included, with a form to add one. */
export function DropStyles({ drop, onOpen }: Props) {
  const [styles, setStyles] = useState<Style[]>([]);
  const [name, setName] = useState("");
  const [color, setColor] = useState("");
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    styleApi.dropStyles(drop.id).then(setStyles, (raw) => setError(toCoreError(raw)));
  }, [drop.id]);
  useEffect(reload, [reload]);

  async function add(e: FormEvent) {
    e.preventDefault();
    setError(undefined);
    try {
      await styleApi.createStyle(drop.id, name, color);
      setName("");
      setColor("");
    } catch (raw) {
      setError(toCoreError(raw));
    }
    reload();
  }

  return (
    <div className="drop-styles">
      <h3>Styles</h3>
      {error && <p className="error">{error.message}</p>}
      {styles.length === 0 ? (
        <p className="empty">No Styles yet.</p>
      ) : (
        <ul>
          {styles.map((s) => (
            <li key={s.id}>
              <button className={s.status === "cut" ? "style-row cut" : "style-row"} onClick={() => onOpen(s.id)}>
                <span>
                  {s.name} <span className="color">· {s.color}</span>
                </span>
                <span className="status">{styleStatusLabel[s.status]}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
      <form className="style-create" onSubmit={add}>
        <input placeholder="Style name" value={name} onChange={(e) => setName(e.target.value)} />
        <input placeholder="Color" value={color} onChange={(e) => setColor(e.target.value)} />
        <button type="submit" disabled={!name.trim() || !color.trim()}>
          Add Style
        </button>
      </form>
    </div>
  );
}
