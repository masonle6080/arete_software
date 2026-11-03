import { useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { thesisApi } from "./api";
import { ALL_THESES, areaOption, NEW_THESIS, pickerChoice } from "./pickerChoice";
import { statusLabel, type ThesisGroup } from "./types";

interface Props {
  /** The current Thesis, or undefined when none is picked. */
  value: number | undefined;
  /** Change this to reload the list, for example after a Thesis is added. */
  version: number;
  /** Called with undefined when the current Thesis is no longer in the library. */
  onPick(id: number | undefined): void;
  /** The Shared areas, listed after the Theses. */
  sharedAreas: { id: string; label: string }[];
  /** The Shared area that is open, if any. The picker shows it in place of the Thesis. */
  openArea?: string;
  /** Called when a Shared area is picked. */
  onArea(id: string): void;
  /** "New Thesis…" at the end of the list. */
  onNewThesis(): void;
  /** "All Theses…" at the end of the list. */
  onAllTheses(): void;
}

/**
 * The sidebar's choice of current Thesis, grouped by status, then the Shared
 * areas. It ends with "New Thesis…" and "All Theses…", which act and leave
 * the choice as it was.
 */
export function ThesisPicker({ value, version, sharedAreas, openArea, onArea, onPick, onNewThesis, onAllTheses }: Props) {
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
        value={openArea !== undefined ? areaOption(openArea) : (value ?? "")}
        onChange={(e) => {
          const choice = pickerChoice(e.target.value);
          if (choice?.kind === "thesis") onPick(choice.id);
          else if (choice?.kind === "area") onArea(choice.id);
          else if (choice?.kind === "new") onNewThesis();
          else if (choice?.kind === "all") onAllTheses();
        }}
      >
        {value === undefined && openArea === undefined && <option value="">Pick a Thesis…</option>}
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
        <optgroup label="Shared">
          {sharedAreas.map((a) => (
            <option key={a.id} value={areaOption(a.id)}>
              {a.label}
            </option>
          ))}
        </optgroup>
        <option disabled>──────────</option>
        <option value={NEW_THESIS}>New Thesis…</option>
        <option value={ALL_THESES}>All Theses…</option>
      </select>
      {error && <p className="error">{error.message}</p>}
    </div>
  );
}
