import { steps } from "../../../shell/navPath";
import { ThesisStyles } from "../../style/ThesisStyles";
import type { ThesisSection, ThesisSectionProps } from ".";

function StylesSectionView({ thesis, onOpen }: ThesisSectionProps) {
  return <ThesisStyles thesis={thesis} onOpen={(style) => onOpen(steps.style(style))} />;
}

export const stylesSection: ThesisSection = { id: "styles", title: "Styles", Component: StylesSectionView };
