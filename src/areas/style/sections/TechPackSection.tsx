import { TechPackEditor } from "../../techPack/TechPackEditor";
import type { StyleSection } from ".";

/** The Tech Pack section of the Style view. The Tech Pack area owns the editor. */
export const techPackSection: StyleSection = { id: "tech-pack", title: "Tech Pack", Component: TechPackEditor };
