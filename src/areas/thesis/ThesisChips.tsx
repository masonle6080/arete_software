import { useEffect, useState } from "react";
import { toCoreError } from "../../shell/coreError";
import { steps, type Step } from "../../shell/navPath";
import { listenTreeChanges } from "../../shell/treeChanges";
import { Chip, countLabel } from "../../ui";
import { documentaryStatusLabel } from "../documentary/types";
import { stageLabel } from "../drop/types";
import { linksApi, type ThesisLinks } from "../links/api";
import type { Thesis } from "./types";

interface Props {
  /** The Thesis as last loaded. The chips reload each time it reloads. */
  thesis: Thesis;
  onOpen(step: Step): void;
}

/** The Thesis header chips: Drop status · N Essays · Documentary status. Each opens its tab. */
export function ThesisChips({ thesis, onOpen }: Props) {
  const [links, setLinks] = useState<ThesisLinks>();
  const [error, setError] = useState<string>();
  // Linking an Essay announces a tree change; the Essay count follows it.
  const [changes, setChanges] = useState(0);
  useEffect(() => listenTreeChanges(() => setChanges((n) => n + 1)), []);

  useEffect(() => {
    let live = true;
    linksApi.thesis(thesis.id).then(
      (l) => live && setLinks(l),
      (raw) => live && setError(toCoreError(raw).message),
    );
    return () => {
      live = false;
    };
  }, [thesis, changes]);

  if (error) return <p className="error">{error}</p>;
  if (!links) return null;
  const { drop, essayCount, documentary } = links;
  return (
    <div className="chips header-chips">
      <Chip onClick={() => onOpen(steps.thesis("drop"))}>{drop ? `Drop · ${stageLabel[drop.stage]}` : "No Drop"}</Chip>
      <Chip onClick={() => onOpen(steps.thesis("essays"))}>{countLabel(essayCount, "Essay")}</Chip>
      <Chip onClick={() => onOpen(steps.thesis("documentary"))}>
        {documentary ? `Documentary · ${documentaryStatusLabel[documentary.status]}` : "No Documentary"}
      </Chip>
    </div>
  );
}
