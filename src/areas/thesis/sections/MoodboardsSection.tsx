import { Moodboards } from "../../moodboard/Moodboards";
import type { ThesisSection, ThesisSectionProps } from "./index";

function ThesisMoodboards({ thesis, onOpenMoodboard }: ThesisSectionProps) {
  return <Moodboards owner={{ kind: "thesis", id: thesis.id }} onOpen={onOpenMoodboard} />;
}

export const moodboardsSection: ThesisSection = {
  id: "moodboards",
  title: "Moodboards",
  Component: ThesisMoodboards,
};
