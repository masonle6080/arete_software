import { documentaryStatusLabel, type DocumentaryStatus } from "../areas/documentary/types";
import { orderStatusLabel, type ProductionOrderStatus } from "../areas/productionOrder/api";
import { sampleOutcomeLabel, type SampleOutcome } from "../areas/sample/api";
import { styleStatusLabel, type StyleStatus } from "../areas/style/types";
import { statusLabel, type ThesisStatus } from "../areas/thesis/types";

/**
 * The four colors a status Badge can have:
 * - gray: not started, or a draft
 * - accent: in progress
 * - green: done or approved
 * - red (struck through): cut, cancelled, abandoned, or rejected
 */
export type BadgeGroup = "gray" | "accent" | "green" | "red";

/** The status of each lifecycle that the app shows as a Badge. */
export interface StatusOf {
  thesis: ThesisStatus;
  style: StyleStatus;
  documentary: DocumentaryStatus;
  productionOrder: ProductionOrderStatus;
  sample: SampleOutcome;
}

export type Lifecycle = keyof StatusOf;

const groups: { [L in Lifecycle]: Record<StatusOf[L], BadgeGroup> } = {
  thesis: {
    seed: "gray",
    developing: "accent",
    design: "accent",
    sampling: "accent",
    production: "accent",
    released: "green",
    archived: "green",
    abandoned: "red",
  },
  style: { draft: "gray", sampling: "accent", approved: "green", cut: "red" },
  documentary: { planning: "gray", filming: "accent", editing: "accent", released: "green", abandoned: "red" },
  productionOrder: {
    draft: "gray",
    placed: "accent",
    inProduction: "accent",
    shipped: "accent",
    received: "green",
    cancelled: "red",
  },
  // A Pending Sample is still with the Vendor. A Revised one led to the next round.
  sample: { pending: "accent", revised: "gray", approved: "green", rejected: "red" },
};

const labels: { [L in Lifecycle]: Record<StatusOf[L], string> } = {
  thesis: statusLabel,
  style: styleStatusLabel,
  documentary: documentaryStatusLabel,
  productionOrder: orderStatusLabel,
  sample: sampleOutcomeLabel,
};

/** The color group of `status` in `lifecycle`. */
export function badgeGroup<L extends Lifecycle>(lifecycle: L, status: StatusOf[L]): BadgeGroup {
  return groups[lifecycle][status];
}

/** The founder-facing name of `status` in `lifecycle`. */
export function badgeLabel<L extends Lifecycle>(lifecycle: L, status: StatusOf[L]): string {
  return labels[lifecycle][status];
}
