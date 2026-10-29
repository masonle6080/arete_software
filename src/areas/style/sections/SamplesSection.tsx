import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import { sampleApi, sampleOutcomeLabel, type Sample, type SampleOutcome } from "../../sample/api";
import { vendorApi, type Vendor } from "../../vendor/api";
import { NotesPanel } from "../../note/NotesPanel";
import { FitNotes } from "./FitNotes";
import type { StyleSection, StyleSectionProps } from ".";

const DECISIONS: SampleOutcome[] = ["approved", "revised", "rejected"];

/** Sample rounds of the Style, by Vendor. Revising a Sample creates the next round. */
function SamplesSectionView({ style }: StyleSectionProps) {
  const [samples, setSamples] = useState<Sample[]>([]);
  const [vendors, setVendors] = useState<Vendor[]>([]);
  const [vendorId, setVendorId] = useState<number>();
  const [openId, setOpenId] = useState<number>();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    sampleApi.styleSamples(style.id).then(setSamples, (raw) => setError(toCoreError(raw)));
  }, [style.id]);
  useEffect(reload, [reload]);
  useEffect(() => {
    vendorApi.listVendors().then(setVendors, (raw) => setError(toCoreError(raw)));
  }, []);

  async function run(action: () => Promise<unknown>) {
    setError(undefined);
    try {
      await action();
      reload();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  async function request() {
    if (vendorId === undefined) return;
    await run(async () => setOpenId((await sampleApi.requestSample(style.id, vendorId)).id));
  }

  const vendorName = (id: number) => vendors.find((v) => v.id === id)?.name ?? `Vendor ${id}`;
  const opened = samples.find((s) => s.id === openId);

  return (
    <div className="style-samples">
      {error && <p className="error">{error.message}</p>}
      {samples.length === 0 ? (
        <p className="empty">No Samples yet.</p>
      ) : (
        <ul>
          {samples.map((s) => (
            <li key={s.id}>
              <button className={s.id === openId ? "sample-row open" : "sample-row"} onClick={() => setOpenId(s.id === openId ? undefined : s.id)}>
                <span>
                  {vendorName(s.vendorId)} · round {s.round}
                </span>
                <span className="outcome">{sampleOutcomeLabel[s.outcome]}</span>
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="sample-request">
        <select value={vendorId ?? ""} onChange={(e) => setVendorId(e.target.value ? Number(e.target.value) : undefined)}>
          <option value="">Choose a Vendor</option>
          {vendors.map((v) => (
            <option key={v.id} value={v.id}>
              {v.name}
            </option>
          ))}
        </select>
        <button onClick={request} disabled={vendorId === undefined}>
          Request Sample
        </button>
      </div>
      {opened && <SampleDetail key={opened.id} sample={opened} vendorName={vendorName(opened.vendorId)} run={run} />}
    </div>
  );
}

interface DetailProps {
  sample: Sample;
  vendorName: string;
  run(action: () => Promise<unknown>): Promise<void>;
}

/** One Sample: photos, feedback, fit notes (Tech Pack Styles only), outcome, and Notes. */
function SampleDetail({ sample, vendorName, run }: DetailProps) {
  const [feedback, setFeedback] = useState(sample.feedback);

  async function attachPhotos() {
    const picked = await open({ multiple: true, title: "Attach Sample photos" });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    await run(() => sampleApi.attachSamplePhotos(sample.id, paths));
  }

  return (
    <div className="sample-detail">
      <h3>
        {vendorName} · round {sample.round} · {sampleOutcomeLabel[sample.outcome]}
      </h3>
      <h4>Photos</h4>
      {sample.photos.length === 0 ? (
        <p className="empty">No photos yet.</p>
      ) : (
        <ul>
          {sample.photos.map((p) => (
            <li key={p.id} title={p.path}>
              {p.fileName}
            </li>
          ))}
        </ul>
      )}
      <button onClick={attachPhotos}>Attach photos</button>
      <FitNotes sampleId={sample.id} />
      <h4>Feedback</h4>
      <textarea rows={4} value={feedback} onChange={(e) => setFeedback(e.target.value)} />
      <div className="actions">
        <button onClick={() => run(() => sampleApi.setSampleFeedback(sample.id, feedback))} disabled={feedback === sample.feedback}>
          Save feedback
        </button>
      </div>
      {sample.outcome === "pending" && (
        <div className="actions">
          {DECISIONS.map((outcome) => (
            <button key={outcome} onClick={() => run(() => sampleApi.decideSample(sample.id, outcome))}>
              Mark {sampleOutcomeLabel[outcome].toLowerCase()}
            </button>
          ))}
        </div>
      )}
      <NotesPanel owner={{ kind: "sample", id: sample.id }} />
    </div>
  );
}

export const samplesSection: StyleSection = { id: "samples", title: "Samples", Component: SamplesSectionView };
