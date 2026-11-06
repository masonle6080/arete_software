import { describe, expect, it } from "vitest";
import { documentaryStatusLabel } from "../areas/documentary/types";
import { orderStatusLabel } from "../areas/productionOrder/api";
import { sampleOutcomeLabel } from "../areas/sample/api";
import { styleStatusLabel } from "../areas/style/types";
import { statusLabel } from "../areas/thesis/types";
import { badgeGroup, type BadgeGroup, type Lifecycle } from "./badge";

/** The color group each status should show, decided from spec #34. */
const expected: Record<Lifecycle, Record<string, BadgeGroup>> = {
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
  sample: { pending: "accent", revised: "gray", approved: "green", rejected: "red" },
};

/** Every status the app knows, taken from each lifecycle's label table. */
const allStatuses: Record<Lifecycle, string[]> = {
  thesis: Object.keys(statusLabel),
  style: Object.keys(styleStatusLabel),
  documentary: Object.keys(documentaryStatusLabel),
  productionOrder: Object.keys(orderStatusLabel),
  sample: Object.keys(sampleOutcomeLabel),
};

describe("badgeGroup", () => {
  for (const lifecycle of Object.keys(allStatuses) as Lifecycle[]) {
    it(`maps every ${lifecycle} status to its color group`, () => {
      const statuses = allStatuses[lifecycle];
      expect(statuses.length).toBeGreaterThan(0);
      for (const status of statuses) {
        expect([status, badgeGroup(lifecycle, status as never)]).toEqual([status, expected[lifecycle][status]]);
      }
    });
  }

  it("expects exactly the statuses each lifecycle has", () => {
    for (const lifecycle of Object.keys(expected) as Lifecycle[]) {
      expect(Object.keys(expected[lifecycle]).sort()).toEqual([...allStatuses[lifecycle]].sort());
    }
  });
});
