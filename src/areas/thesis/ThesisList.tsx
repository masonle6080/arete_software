import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { thesisApi } from "./api";
import { statusLabel, type ThesisGroup } from "./types";

interface Props {
  onOpen(id: number): void;
}

/** Capture form plus every Thesis, grouped by status. */
export function ThesisList({ onOpen }: Props) {
  const [groups, setGroups] = useState<ThesisGroup[]>([]);
  const [title, setTitle] = useState("");
  const [statement, setStatement] = useState("");
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    thesisApi.listTheses().then(setGroups, (raw) => setError(toCoreError(raw)));
  }, []);
  useEffect(reload, [reload]);

  async function create(e: FormEvent) {
    e.preventDefault();
    setError(undefined);
    try {
      await thesisApi.createThesis(title, statement);
      setTitle("");
      setStatement("");
      reload();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <section className="thesis-list">
      <h1>Theses</h1>
      <form className="thesis-capture" onSubmit={create}>
        <input placeholder="Title" value={title} onChange={(e) => setTitle(e.target.value)} />
        <textarea
          placeholder="Short statement"
          rows={2}
          value={statement}
          onChange={(e) => setStatement(e.target.value)}
        />
        <button type="submit" disabled={!title.trim()}>
          Add Thesis
        </button>
      </form>
      {error && <p className="error">{error.message}</p>}

      {groups.map((group) => (
        <div key={group.status} className="thesis-group">
          <h2>
            {statusLabel[group.status]} <span className="count">{group.theses.length}</span>
          </h2>
          {group.theses.length === 0 ? (
            <p className="empty">None</p>
          ) : (
            <ul>
              {group.theses.map((t) => (
                <li key={t.id}>
                  <button className="thesis-row" onClick={() => onOpen(t.id)}>
                    <strong>{t.title}</strong>
                    {t.statement && <span className="statement">{t.statement}</span>}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </section>
  );
}
