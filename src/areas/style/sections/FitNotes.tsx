import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { sampleApi, type FitCell, type SampleFit } from "../../sample/api";
import "./fitNotes.css";

/** Parses a measured value. Empty text clears it (`null`); other text must be a number. */
function parseMeasured(text: string): number | null | undefined {
  const t = text.trim();
  if (t === "") return null;
  const n = Number(t);
  return Number.isFinite(n) ? n : undefined;
}

function signed(n: number): string {
  const rounded = Math.round(n * 100) / 100;
  return rounded > 0 ? `+${rounded}` : `${rounded}`;
}

/**
 * Fit notes of one Sample: per Point of Measure and size, the Tech Pack
 * value, the measured value, and how far off it is against the tolerance.
 * Shows nothing when the Style is not built on a Tech Pack.
 */
export function FitNotes({ sampleId }: { sampleId: number }) {
  const [fit, setFit] = useState<SampleFit | null>();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    sampleApi.sampleFit(sampleId).then(setFit, (raw) => setError(toCoreError(raw)));
  }, [sampleId]);
  useEffect(reload, [reload]);

  async function commit(pointOfMeasureId: number, cell: FitCell, text: string) {
    const measured = parseMeasured(text);
    if (measured === undefined) {
      setError({ kind: "InvalidInput", message: "A measured value must be a number." });
      return;
    }
    setError(undefined);
    try {
      setFit(await sampleApi.setFitNote(sampleId, pointOfMeasureId, cell.sizeId, measured));
    } catch (raw) {
      setError(toCoreError(raw));
      reload();
    }
  }

  if (fit === undefined) return error ? <p className="error">{error.message}</p> : null;
  if (fit === null) return null;

  return (
    <div className="fit-notes">
      <h4>Fit notes ({fit.unit})</h4>
      {error && <p className="error">{error.message}</p>}
      {fit.rows.length === 0 || fit.sizes.length === 0 ? (
        <p className="empty">Add Points of Measure and sizes to the Tech Pack to record fit notes.</p>
      ) : (
        <div className="fit-scroll">
          <table className="fit-table">
            <thead>
              <tr>
                <th>Point of Measure</th>
                <th>Tol. ±</th>
                {fit.sizes.map((z) => (
                  <th key={z.id}>{z.name}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {fit.rows.map((row) => (
                <tr key={row.pointOfMeasureId}>
                  <th scope="row">{row.name}</th>
                  <td>{row.tolerance}</td>
                  {row.cells.map((cell, i) => {
                    const size = fit.sizes[i].name;
                    const value = cell.measured === null ? "" : String(cell.measured);
                    const state = cell.withinTolerance === null ? "" : cell.withinTolerance ? "within" : "outside";
                    return (
                      <td key={cell.sizeId} className={`fit-cell ${state}`}>
                        <span className="spec" title="Tech Pack value">
                          {cell.spec ?? "–"}
                        </span>
                        <input
                          key={value}
                          className="measured"
                          aria-label={`Measured ${row.name}, size ${size}`}
                          placeholder="measured"
                          defaultValue={value}
                          onBlur={(e) => {
                            if (e.currentTarget.value !== value) commit(row.pointOfMeasureId, cell, e.currentTarget.value);
                          }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") e.currentTarget.blur();
                          }}
                        />
                        {cell.deviation !== null && (
                          <span
                            className="deviation"
                            title={cell.withinTolerance ? "Within tolerance" : "Outside tolerance"}
                          >
                            {signed(cell.deviation)}
                          </span>
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
