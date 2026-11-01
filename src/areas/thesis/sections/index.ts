import type { ComponentType } from "react";
import type { Step } from "../../../shell/navPath";
import type { Thesis } from "../types";
import { dropSection } from "./DropSection";
import { essaysSection } from "./EssaysSection";
import { moodboardsSection } from "./MoodboardsSection";
import { documentarySection } from "./DocumentarySection";

/** Props every Thesis view section receives. */
export interface ThesisSectionProps {
  thesis: Thesis;
  /** Call after a section changes the Thesis itself (for example, its status). */
  onThesisChanged(): void;
  /**
   * Opens an item (a Style, an Essay, a Moodboard, or the Script) one level
   * below the tab in the navigation path, so it stays inside this Thesis.
   */
  onOpen(step: Step): void;
}

/** One section of the Thesis view, for example Essays or Drop. Each is a sidebar tab. */
export interface ThesisSection {
  id: string;
  title: string;
  Component: ComponentType<ThesisSectionProps>;
}

/**
 * The sections of the Thesis view, in display order. To add one (Essays,
 * Moodboards, Drop, Documentary), create `sections/<Name>Section.tsx` that
 * exports a `ThesisSection`, then add one import and one line here. The view
 * renders the section of the open tab under its title; the section owns its
 * own data.
 */
export const thesisSections: ThesisSection[] = [
  essaysSection,
  moodboardsSection,
  dropSection,
  documentarySection,
];
