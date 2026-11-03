import type { ComponentType } from "react";
import type { Step, ThesisTab } from "../../../shell/navPath";
import type { Thesis } from "../types";
import { stylesSection } from "./StylesSection";
import { productionSection } from "./ProductionSection";
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

/** The Thesis tabs that show a section: every tab but Overview. */
export type SectionTab = Exclude<ThesisTab, "overview">;

/** One section of the Thesis view, for example Essays or Styles. Each is a sidebar tab. */
export interface ThesisSection {
  id: SectionTab;
  title: string;
  Component: ComponentType<ThesisSectionProps>;
}

/**
 * The section of each Thesis tab but Overview. To add one, add its tab to
 * `thesisTabs` (`shell/navPath.ts`, which sets the tab order), create
 * `sections/<Name>Section.tsx` that exports a `ThesisSection`, then add one
 * import and one line here. The view renders the section of the open tab
 * under its title; the section owns its own data.
 */
export const thesisSections: Record<SectionTab, ThesisSection> = {
  essays: essaysSection,
  moodboards: moodboardsSection,
  styles: stylesSection,
  documentary: documentarySection,
  production: productionSection,
};
