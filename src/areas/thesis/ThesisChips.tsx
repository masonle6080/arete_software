import { useEffect } from "react";
import { useCoreQuery } from "../../shell/coreQuery";
import { steps, type Step } from "../../shell/navPath";
import { listenTreeChanges } from "../../shell/treeChanges";
import { Chip, countLabel } from "../../ui";
import { documentaryStatusLabel } from "../documentary/types";
import { stageLabel } from "../drop/types";
import { linksApi } from "../links/api";
import type { Thesis } from "./types";

interface Props {
  /** The Thesis as last loaded. The chips reload each time it reloads. */
  thesis: Thesis;
  onOpen(step: Step): void;
}

/** The Thesis header chips: Drop status · N Essays · Documentary status. Each opens its tab. */
export function ThesisChips({ thesis, onOpen }: Props) {
  const { data: links, error, reload } = useCoreQuery(() => linksApi.thesis(thesis.id), [thesis]);
  // Linking an Essay announces a tree change; the Essay count follows it.
  useEffect(() => listenTreeChanges(reload), [reload]);

  if (error) return <p className="error">{error.message}</p>;
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
