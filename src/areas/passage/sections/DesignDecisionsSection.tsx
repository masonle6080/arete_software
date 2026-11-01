import { useEffect, useState } from "react";
import { toCoreError } from "../../../shell/coreError";
import { designAspectLabel, designDecisionApi, type PassageDesignDecision } from "../../designDecision/api";
import type { PassageSection, PassageSectionProps } from ".";
import { Badge, EmptyText } from "../../../ui";

/** The Design Decisions this Passage motivates, on any Style (Cut ones too). */
function DesignDecisionsSection({ passage }: PassageSectionProps) {
  const [decisions, setDecisions] = useState<PassageDesignDecision[]>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    designDecisionApi.ofPassage(passage.id).then(setDecisions, (e) => setError(toCoreError(e).message));
  }, [passage.id]);

  if (error) return <p className="error">{error}</p>;
  if (!decisions) return null;
  if (decisions.length === 0) return <EmptyText>No Design Decisions linked yet.</EmptyText>;
  return (
    <ul className="passage-design-decisions">
      {decisions.map((d) => (
        <li key={d.id}>
          <p className="meta">
            {d.styleName} · {d.styleColor} <Badge lifecycle="style" status={d.styleStatus} /> · {designAspectLabel[d.aspect]}
          </p>
          <p className="rationale">{d.rationale}</p>
        </li>
      ))}
    </ul>
  );
}

export const designDecisionsSection: PassageSection = {
  id: "design-decisions",
  title: "Design Decisions",
  Component: DesignDecisionsSection,
};
