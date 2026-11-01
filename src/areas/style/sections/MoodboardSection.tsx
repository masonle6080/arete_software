import { steps } from "../../../shell/navPath";
import { Moodboards } from "../../moodboard/Moodboards";
import type { StyleSection, StyleSectionProps } from ".";

/** The Style's optional single Moodboard, on the same canvas as a Thesis Moodboard. */
function MoodboardSectionView({ style, onOpen }: StyleSectionProps) {
  return <Moodboards owner={{ kind: "style", id: style.id }} single onOpen={(board) => onOpen(steps.moodboard(board))} />;
}

export const moodboardSection: StyleSection = {
  id: "moodboard",
  title: "Moodboard",
  segment: "design",
  Component: MoodboardSectionView,
};
