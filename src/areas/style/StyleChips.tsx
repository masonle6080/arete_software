import { useCoreQuery } from "../../shell/coreQuery";
import { steps, type Step } from "../../shell/navPath";
import { Chip, countLabel } from "../../ui";
import { blankLabel } from "../blank/api";
import { linksApi } from "../links/api";
import type { StyleSegment } from "./segments";
import type { Style } from "./types";

interface Props {
  /** The Style as last loaded. The chips reload each time it reloads. */
  style: Style;
  onOpen(step: Step): void;
  /** Opens a segment of this Style screen. */
  onSegment(segment: StyleSegment): void;
}

/**
 * The Style header chips: its Thesis and Drop, its base (a Blank or the
 * Tech Pack), and the number of Passages that motivate it. The Thesis and
 * Drop chips open them; the base chip opens Build, where the base is; the
 * Passage chip opens Design, where the Design Decisions show their Passages.
 */
export function StyleChips({ style, onOpen, onSegment }: Props) {
  const { data: links, error } = useCoreQuery(() => linksApi.style(style.id), [style]);

  if (error) return <p className="error">{error.message}</p>;
  if (!links) return null;
  const { thesis, drop, base, passageCount } = links;
  return (
    <div className="chips header-chips">
      <Chip title="Open the Thesis" onClick={() => onOpen(steps.thesis("overview", thesis.id))}>
        {thesis.title}
      </Chip>
      <Chip title="Open Styles" onClick={() => onOpen(steps.thesis("styles", thesis.id))}>
        Drop · {drop.name}
      </Chip>
      <Chip title="Open Build" onClick={() => onSegment("build")}>
        {base === null ? "No base" : base.kind === "blank" ? `Base: ${blankLabel(base.blank)}` : "Base: Tech Pack"}
      </Chip>
      <Chip title="Open Design" onClick={() => onSegment("design")}>
        {countLabel(passageCount, "Passage")}
      </Chip>
    </div>
  );
}
