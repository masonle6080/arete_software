import { useEffect, useState } from "react";
import { toCoreError } from "../../shell/coreError";
import { passageApi, type Passage } from "./api";
import "./passage.css";

interface Props {
  documentId: number;
  /** Changes after each save, so the list reloads. */
  version: number;
  onOpen(id: string): void;
}

/**
 * The orphaned Passages of a document: Passages whose text was fully
 * deleted. Shows nothing when there are none.
 */
export function OrphanedPassages({ documentId, version, onOpen }: Props) {
  const [orphans, setOrphans] = useState<Passage[]>([]);
  const [error, setError] = useState<string>();

  useEffect(() => {
    passageApi.ofDocument(documentId).then(
      (all) => setOrphans(all.filter((p) => p.orphaned)),
      (e) => setError(toCoreError(e).message),
    );
  }, [documentId, version]);

  if (error) return <p className="error">{error}</p>;
  if (orphans.length === 0) return null;
  return (
    <details className="orphaned-passages">
      <summary>Orphaned Passages ({orphans.length})</summary>
      <ul>
        {orphans.map((p) => (
          <li key={p.id}>
            <button onClick={() => onOpen(p.id)}>“{p.lastKnownText}”</button>
          </li>
        ))}
      </ul>
    </details>
  );
}
