import { Moodboards } from "../../moodboard/Moodboards";
import type { StyleSection, StyleSectionProps } from ".";

/** The Style's optional single Moodboard, on the same canvas as a Thesis Moodboard. */
function MoodboardSectionView({ style, onOpenMoodboard }: StyleSectionProps) {
  return <Moodboards owner={{ kind: "style", id: style.id }} single onOpen={onOpenMoodboard} />;
}

export const moodboardSection: StyleSection = { id: "moodboard", title: "Moodboard", Component: MoodboardSectionView };
