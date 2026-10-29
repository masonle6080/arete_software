import { call } from "../../shell/invoke";
import type { TechPackSize } from "../techPack/api";

/** Mirrors `SampleOutcome` in `src-tauri/src/sample/mod.rs`. */
export type SampleOutcome = "pending" | "approved" | "revised" | "rejected";

export const sampleOutcomeLabel: Record<SampleOutcome, string> = {
  pending: "Pending",
  approved: "Approved",
  revised: "Revised",
  rejected: "Rejected",
};

/** Mirrors `SamplePhoto`: one photo of a Sample. */
export interface SamplePhoto {
  id: number;
  media: string;
  /** The name the file had when it was attached. */
  fileName: string;
  /** Absolute path of the stored copy in the library. */
  path: string;
}

/** Mirrors `Sample` in `src-tauri/src/sample/mod.rs`. */
export interface Sample {
  id: number;
  styleId: number;
  vendorId: number;
  /** The round number for this Style and Vendor, from 1. */
  round: number;
  outcome: SampleOutcome;
  feedback: string;
  photos: SamplePhoto[];
}

/** Mirrors `FitCell`: one Point of Measure in one size on a Sample. */
export interface FitCell {
  sizeId: number;
  /** The Tech Pack value, or null when the cell is empty. */
  spec: number | null;
  /** The value measured on the Sample, or null when not recorded. */
  measured: number | null;
  /** `measured - spec`, when both are known. */
  deviation: number | null;
  withinTolerance: boolean | null;
}

/** Mirrors `FitRow`: one Point of Measure on a Sample. */
export interface FitRow {
  pointOfMeasureId: number;
  name: string;
  tolerance: number;
  /** One per size, in the order of `SampleFit.sizes`. */
  cells: FitCell[];
}

/** Mirrors `SampleFit` in `src-tauri/src/sample/fit.rs`: a Sample's fit notes. */
export interface SampleFit {
  sampleId: number;
  unit: string;
  sizes: TechPackSize[];
  rows: FitRow[];
}

/** IPC wrappers for `src-tauri/src/ipc/sample.rs`. */
export const sampleApi = {
  requestSample: (styleId: number, vendorId: number) => call<Sample>("request_sample", { styleId, vendorId }),
  styleSamples: (styleId: number) => call<Sample[]>("style_samples", { styleId }),
  setSampleFeedback: (id: number, feedback: string) => call<Sample>("set_sample_feedback", { id, feedback }),
  /** `sources` are paths outside the library; the core imports a copy of each. */
  attachSamplePhotos: (id: number, sources: string[]) => call<SamplePhoto[]>("attach_sample_photos", { id, sources }),
  /** Marking a Sample revised also creates the next round in the core. */
  decideSample: (id: number, outcome: SampleOutcome) => call<Sample>("decide_sample", { id, outcome }),
  /** Null when the Style is not built on a Tech Pack: no fit notes are offered. */
  sampleFit: (id: number) => call<SampleFit | null>("sample_fit", { id }),
  /** `measured` null clears the value. */
  setFitNote: (id: number, pointOfMeasureId: number, sizeId: number, measured: number | null) =>
    call<SampleFit>("set_fit_note", { id, pointOfMeasureId, sizeId, measured }),
};
