import { useEffect, useState } from "react";
import { toCoreError } from "../../shell/coreError";
import { passageApi, type Passage } from "./api";
import { passageSections } from "./sections";
import "./passage.css";
import { Chip, EmptyText, SectionHeader } from "../../ui";

interface Props {
  id: string;
}

/**
 * One Passage: its last known text, its document, whether it is orphaned,
 * then everything linked to it (one registered section per kind of link).
 */
export function PassageView({ id }: Props) {
  const [passage, setPassage] = useState<Passage>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    passageApi.get(id).then(setPassage, (e) => setError(toCoreError(e).message));
  }, [id]);

  const kind = passage?.documentKind === "script" ? "Script" : "Essay";

  return (
    <section className="passage-view">
      {error && <p className="error">{error}</p>}
      {passage && (
        <>
          <header>
            <p className="meta">
              Passage in {kind} “{passage.documentTitle || "Untitled"}”
              {passage.orphaned && <Chip>Orphaned</Chip>}
            </p>
            <blockquote>{passage.lastKnownText}</blockquote>
            {passage.orphaned && (
              <p className="muted">
                Its text was deleted. It keeps its last known text and its links. Undo the deletion to restore it.
              </p>
            )}
          </header>
          {passageSections.length === 0 && <EmptyText>Nothing is linked to this Passage yet.</EmptyText>}
          {passageSections.map(({ id: sectionId, title, Component }) => (
            <section key={sectionId} className="passage-section">
              <SectionHeader title={title} />
              <Component passage={passage} />
            </section>
          ))}
        </>
      )}
    </section>
  );
}
