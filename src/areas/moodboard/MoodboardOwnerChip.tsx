import { useCoreQuery } from "../../shell/coreQuery";
import { steps, type Step } from "../../shell/navPath";
import { Chip } from "../../ui";
import { styleApi } from "../style/api";
import { thesisApi } from "../thesis/api";
import type { MoodboardOwner } from "./types";

interface Props {
  owner: MoodboardOwner;
  onOpen(step: Step): void;
}

/** A chip for the Thesis or Style a Moodboard belongs to. It opens that Thesis or Style. */
export function MoodboardOwnerChip({ owner, onOpen }: Props) {
  const { data: chip, error } = useCoreQuery<{ label: string; step: Step }>(
    () =>
      owner.kind === "thesis"
        ? thesisApi.getThesis(owner.id).then((t) => ({ label: t.title, step: steps.thesis("overview", t.id) }))
        : styleApi.getStyle(owner.id).then((s) => ({ label: steps.style(s).label, step: steps.style(s) })),
    [owner.kind, owner.id],
  );

  if (error) return <span className="error">{error.message}</span>;
  if (!chip) return null;
  return (
    <Chip title={owner.kind === "thesis" ? "Open the Thesis" : "Open the Style"} onClick={() => onOpen(chip.step)}>
      {chip.label}
    </Chip>
  );
}
