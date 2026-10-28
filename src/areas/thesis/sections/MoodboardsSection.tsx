import { Moodboards } from "../../moodboard/Moodboards";
import type { ThesisSection, ThesisSectionProps } from "./index";

function ThesisMoodboards({ thesis }: ThesisSectionProps) {
  return <Moodboards owner={{ kind: "thesis", id: thesis.id }} />;
}

export const moodboardsSection: ThesisSection = {
  id: "moodboards",
  title: "Moodboards",
  Component: ThesisMoodboards,
};
