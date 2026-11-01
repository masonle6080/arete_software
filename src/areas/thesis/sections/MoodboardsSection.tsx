import { steps } from "../../../shell/navPath";
import { Moodboards } from "../../moodboard/Moodboards";
import type { ThesisSection, ThesisSectionProps } from "./index";

function ThesisMoodboards({ thesis, onOpen }: ThesisSectionProps) {
  return <Moodboards owner={{ kind: "thesis", id: thesis.id }} onOpen={(board) => onOpen(steps.moodboard(board))} />;
}

export const moodboardsSection: ThesisSection = {
  id: "moodboards",
  title: "Moodboards",
  Component: ThesisMoodboards,
};
