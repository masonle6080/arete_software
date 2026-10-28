import { call } from "../../shell/invoke";

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

/** IPC wrappers for `src-tauri/src/ipc/sample.rs`. */
export const sampleApi = {
  requestSample: (styleId: number, vendorId: number) => call<Sample>("request_sample", { styleId, vendorId }),
  styleSamples: (styleId: number) => call<Sample[]>("style_samples", { styleId }),
  setSampleFeedback: (id: number, feedback: string) => call<Sample>("set_sample_feedback", { id, feedback }),
  /** `sources` are paths outside the library; the core imports a copy of each. */
  attachSamplePhotos: (id: number, sources: string[]) => call<SamplePhoto[]>("attach_sample_photos", { id, sources }),
  /** Marking a Sample revised also creates the next round in the core. */
  decideSample: (id: number, outcome: SampleOutcome) => call<Sample>("decide_sample", { id, outcome }),
};
