import type { ComponentType } from "react";
import type { Thesis } from "../types";
import { dropSection } from "./DropSection";
import { essaysSection } from "./EssaysSection";
import { moodboardsSection } from "./MoodboardsSection";

/** Props every Thesis view section receives. */
export interface ThesisSectionProps {
  thesis: Thesis;
  /** Call after a section changes the Thesis itself (for example, its status). */
  onThesisChanged(): void;
  /** Opens the full Style view, in place of the Thesis view. */
  onOpenStyle(styleId: number): void;
}

/** One section of the Thesis view, for example Essays or Drop. */
export interface ThesisSection {
  id: string;
  title: string;
  Component: ComponentType<ThesisSectionProps>;
}

/**
 * The sections of the Thesis view, in display order. To add one (Essays,
 * Moodboards, Drop, Documentary), create `sections/<Name>Section.tsx` that
 * exports a `ThesisSection`, then add one import and one line here. The view
 * renders each section under its title; the section owns its own data.
 */
export const thesisSections: ThesisSection[] = [
  essaysSection,
  moodboardsSection,
  dropSection,
];
