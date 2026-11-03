import { useCoreQuery } from "../../../shell/coreQuery";
import { EmptyText } from "../../../ui";
import { dropApi } from "../../drop/api";
import { DropProductionOrders } from "../../productionOrder/DropProductionOrders";
import type { ThesisSection, ThesisSectionProps } from ".";

/**
 * Stop-gap until the Drop record goes (spec #44): the Production Orders of
 * the Thesis's active Drop.
 */
function ProductionSectionView({ thesis }: ThesisSectionProps) {
  const { data, error } = useCoreQuery(() => dropApi.thesisDrops(thesis.id), [thesis.id]);
  if (error) return <p className="error">{error.message}</p>;
  if (!data) return null;
  const active = data.drops.find((d) => d.stage !== "cancelled");
  return active ? <DropProductionOrders drop={active} /> : <EmptyText>No Production Orders yet.</EmptyText>;
}

export const productionSection: ThesisSection = { id: "production", title: "Production", Component: ProductionSectionView };
