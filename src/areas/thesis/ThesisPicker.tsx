import { useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { thesisApi } from "./api";
import { statusLabel, type ThesisGroup } from "./types";

interface Props {
  /** The current Thesis, or undefined when none is picked. */
  value: number | undefined;
  /** Change this to reload the list, for example after a Thesis is added. */
  version: number;
  /** Called with undefined when the current Thesis is no longer in the library. */
  onPick(id: number | undefined): void;
}

/** The sidebar's choice of current Thesis, grouped by status. */
export function ThesisPicker({ value, version, onPick }: Props) {
  const [groups, setGroups] = useState<ThesisGroup[]>();
  const [error, setError] = useState<CoreError>();

  useEffect(() => {
    thesisApi.listTheses().then(setGroups, (raw) => setError(toCoreError(raw)));
  }, [version]);

  const missing = groups !== undefined && value !== undefined && !groups.some((g) => g.theses.some((t) => t.id === value));
  useEffect(() => {
    if (missing) onPick(undefined);
  }, [missing, onPick]);

  return (
    <div className="thesis-picker">
      <label htmlFor="thesis-picker">Thesis</label>
      <select
        id="thesis-picker"
        value={value ?? ""}
        onChange={(e) => e.target.value && onPick(Number(e.target.value))}
      >
        {value === undefined && <option value="">Pick a Thesis…</option>}
        {groups
          ?.filter((g) => g.theses.length > 0)
          .map((g) => (
            <optgroup key={g.status} label={statusLabel[g.status]}>
              {g.theses.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.title}
                </option>
              ))}
            </optgroup>
          ))}
      </select>
      {error && <p className="error">{error.message}</p>}
    </div>
  );
}
