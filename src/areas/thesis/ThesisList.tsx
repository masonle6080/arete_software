import { useCallback, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { Badge, Button, EmptyText, SectionHeader } from "../../ui";
import { thesisApi } from "./api";
import type { ThesisGroup } from "./types";

interface Props {
  onOpen(id: number): void;
  /** Called after a Thesis is added, so the sidebar's Thesis picker can reload. */
  onChanged?(): void;
  /** Focuses the title field of the "Add Thesis" form, for "New Thesis…". */
  focusCreate?: boolean;
}

/** Capture form plus every Thesis, grouped by status. */
export function ThesisList({ onOpen, onChanged, focusCreate = false }: Props) {
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
      onChanged?.();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <section className="thesis-list">
      <h1>Theses</h1>
      <form className="thesis-capture" onSubmit={create}>
        <input autoFocus={focusCreate} placeholder="Title" value={title} onChange={(e) => setTitle(e.target.value)} />
        <textarea
          placeholder="Short statement"
          rows={2}
          value={statement}
          onChange={(e) => setStatement(e.target.value)}
        />
        <Button type="submit" variant="primary" disabled={!title.trim()}>
          Add Thesis
        </Button>
      </form>
      {error && <p className="error">{error.message}</p>}

      {groups.map((group) => (
        <div key={group.status} className="thesis-group">
          <SectionHeader
            title={<Badge lifecycle="thesis" status={group.status} />}
            count={group.theses.length}
          />
          {group.theses.length === 0 ? (
            <EmptyText>None</EmptyText>
          ) : (
            <ul>
              {group.theses.map((t) => (
                <li key={t.id}>
                  <Button className="thesis-row" onClick={() => onOpen(t.id)}>
                    <strong>{t.title}</strong>
                    {t.statement && <span className="statement">{t.statement}</span>}
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </section>
  );
}
