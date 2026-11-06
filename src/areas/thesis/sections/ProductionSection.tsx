import { ThesisProductionOrders } from "../../productionOrder/ThesisProductionOrders";
import type { ThesisSection, ThesisSectionProps } from ".";

/** The Production Orders of the Thesis and their cost roll-up. */
function ProductionSectionView({ thesis }: ThesisSectionProps) {
  return <ThesisProductionOrders thesis={thesis} />;
}

export const productionSection: ThesisSection = { id: "production", title: "Production", Component: ProductionSectionView };
