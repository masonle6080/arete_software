import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { announceThesisChange } from "../../shell/thesisChanges";
import { Badge, Button, EmptyText } from "../../ui";
import type { Thesis } from "../thesis/types";
import { styleApi } from "./api";
import { cardFace } from "./cover";
import type { Style } from "./types";
import "./style.css";

interface Props {
  thesis: Thesis;
  /** Opens the Style view. */
  onOpen(style: Style): void;
}

/** The Styles of one Thesis as a card grid of Covers, Cut ones included, with a form to add one while the Thesis takes new Styles. */
export function ThesisStyles({ thesis, onOpen }: Props) {
  const [styles, setStyles] = useState<Style[]>([]);
  const [name, setName] = useState("");
  const [color, setColor] = useState("");
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    styleApi.thesisStyles(thesis.id).then(setStyles, (raw) => setError(toCoreError(raw)));
  }, [thesis.id]);
  useEffect(reload, [reload]);

  async function add(e: FormEvent) {
    e.preventDefault();
    setError(undefined);
    try {
      await styleApi.createStyle(thesis.id, name, color);
      setName("");
      setColor("");
      announceThesisChange();
    } catch (raw) {
      setError(toCoreError(raw));
    }
    reload();
  }

  return (
    <div className="thesis-styles">
      {error && <p className="error">{error.message}</p>}
      {styles.length === 0 ? (
        <EmptyText>No Styles yet.</EmptyText>
      ) : (
        <ul className="style-grid">
          {styles.map((s) => (
            <li key={s.id}>
              <button type="button" className="style-card" onClick={() => onOpen(s)}>
                <CardFaceView style={s} />
                <span className="style-card-name">{s.name}</span>
                <span className="style-card-meta">
                  <span className="color">{s.color}</span>
                  <Badge lifecycle="style" status={s.status} />
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {thesis.canAddStyles && (
        <form className="style-create" onSubmit={add}>
          <input placeholder="Style name" value={name} onChange={(e) => setName(e.target.value)} />
          <input placeholder="Color" value={color} onChange={(e) => setColor(e.target.value)} />
          <Button type="submit" disabled={!name.trim() || !color.trim()}>
            Add Style
          </Button>
        </form>
      )}
    </div>
  );
}

/** The top of a Style's card: its Cover small, or a neutral placeholder with its color name. */
export function CardFaceView({ style }: { style: Style }) {
  const face = cardFace(style);
  return face.kind === "cover" ? (
    <img className="style-card-face" src={convertFileSrc(face.path)} alt={`Cover of ${style.name}`} />
  ) : (
    <span className="style-card-face style-card-placeholder">{face.color}</span>
  );
}
