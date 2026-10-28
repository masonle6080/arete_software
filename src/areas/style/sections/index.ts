import type { ComponentType } from "react";
import type { Style } from "../types";
import { filesSection } from "./FilesSection";
import { techPackSection } from "./TechPackSection";

/** Props every Style view section receives. */
export interface StyleSectionProps {
  style: Style;
  /** Call after a section changes the Style itself (for example, its status). */
  onStyleChanged(): void;
}

/** One section of the Style view, for example Design files or Cost Sheet. */
export interface StyleSection {
  id: string;
  title: string;
  Component: ComponentType<StyleSectionProps>;
}

/**
 * The sections of the Style view, in display order. To add one (Design
 * Decisions, Base, Decorations, Moodboard, Samples, Cost Sheet), create
 * `sections/<Name>Section.tsx` that exports a `StyleSection`, then add one
 * import and one line here. The view renders each section under its title;
 * the section owns its own data.
 */
export const styleSections: StyleSection[] = [
  filesSection,
  techPackSection,
];
