import { useEffect, useState } from "react";
import { toCoreError } from "../../../shell/coreError";
import { designAspectLabel, designDecisionApi, type PassageDesignDecision } from "../../designDecision/api";
import { styleStatusLabel } from "../../style/types";
import type { PassageSection, PassageSectionProps } from ".";

/** The Design Decisions this Passage motivates, on any Style (Cut ones too). */
function DesignDecisionsSection({ passage }: PassageSectionProps) {
  const [decisions, setDecisions] = useState<PassageDesignDecision[]>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    designDecisionApi.ofPassage(passage.id).then(setDecisions, (e) => setError(toCoreError(e).message));
  }, [passage.id]);

  if (error) return <p className="error">{error}</p>;
  if (!decisions) return null;
  if (decisions.length === 0) return <p className="muted">No Design Decisions linked yet.</p>;
  return (
    <ul className="passage-design-decisions">
      {decisions.map((d) => (
        <li key={d.id}>
          <p className="meta">
            {d.styleName} · {d.styleColor} · {styleStatusLabel[d.styleStatus]} · {designAspectLabel[d.aspect]}
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
