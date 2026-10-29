import { useCallback, useEffect, useState } from "react";
import { toCoreError, type CoreError } from "../../../shell/coreError";
import {
  designAspectLabel,
  designAspects,
  designDecisionApi,
  type DesignAspect,
  type DesignDecision,
} from "../../designDecision/api";
import { EssayScreen } from "../../essays";
import { documentApi, type DocumentSummary } from "../../essays/api";
import { passageApi, type Passage } from "../../passage/api";
import { StyleTraceExport } from "../StyleTraceExport";
import type { StyleSection, StyleSectionProps } from ".";

/**
 * The Design Decisions of the Style, each with its rationale and linked
 * Passages. A linked Passage opens in its Essay, selected and in view.
 */
function DesignDecisionsSectionView({ style }: StyleSectionProps) {
  const [decisions, setDecisions] = useState<DesignDecision[]>([]);
  const [aspect, setAspect] = useState<DesignAspect>("decoration");
  const [rationale, setRationale] = useState("");
  const [opened, setOpened] = useState<Passage>();
  const [error, setError] = useState<CoreError>();

  const reload = useCallback(() => {
    designDecisionApi.ofStyle(style.id).then(setDecisions, (raw) => setError(toCoreError(raw)));
  }, [style.id]);
  useEffect(reload, [reload]);

  async function run(action: () => Promise<unknown>): Promise<boolean> {
    setError(undefined);
    try {
      await action();
      reload();
      return true;
    } catch (raw) {
      setError(toCoreError(raw));
      return false;
    }
  }

  async function add() {
    if (await run(() => designDecisionApi.add(style.id, aspect, rationale))) setRationale("");
  }

  if (opened) {
    return (
      <EssayScreen
        id={opened.documentId}
        focusPassageId={opened.id}
        backLabel={`← ${style.name}`}
        onClose={() => {
          setOpened(undefined);
          reload();
        }}
      />
    );
  }

  return (
    <div className="design-decisions">
      <StyleTraceExport style={style} />
      {error && <p className="error">{error.message}</p>}
      {decisions.length === 0 ? (
        <p className="empty">No Design Decisions yet.</p>
      ) : (
        <ul>
          {decisions.map((d) => (
            <li key={d.id} className="design-decision">
              <p className="aspect">{designAspectLabel[d.aspect]}</p>
              <p className="rationale">{d.rationale}</p>
              {d.passages.length > 0 && (
                <ul className="linked-passages">
                  {d.passages.map((p) => (
                    <li key={p.id}>
                      <button className="open" title="Open in its Essay" onClick={() => setOpened(p)}>
                        “{p.lastKnownText}”
                        <span className="source">
                          {p.documentTitle || "Untitled"}
                          {p.orphaned && " · orphaned"}
                        </span>
                      </button>
                      <button className="unlink" onClick={() => run(() => designDecisionApi.unlinkPassage(d.id, p.id))}>
                        Unlink
                      </button>
                    </li>
                  ))}
                </ul>
              )}
              <PassagePicker
                exclude={d.passages.map((p) => p.id)}
                onPick={(passageId) => run(() => designDecisionApi.linkPassage(d.id, passageId))}
              />
            </li>
          ))}
        </ul>
      )}
      <div className="design-decision-add">
        <select value={aspect} aria-label="Aspect" onChange={(e) => setAspect(e.target.value as DesignAspect)}>
          {designAspects.map((a) => (
            <option key={a} value={a}>
              {designAspectLabel[a]}
            </option>
          ))}
        </select>
        <textarea
          rows={3}
          placeholder="Why this choice? (required)"
          value={rationale}
          onChange={(e) => setRationale(e.target.value)}
        />
        <button onClick={add} disabled={rationale.trim() === ""}>
          Add Design Decision
        </button>
      </div>
    </div>
  );
}

interface PickerProps {
  /** Passages already linked; the picker leaves them out. */
  exclude: string[];
  onPick(passageId: string): void;
}

/** Choose an Essay, then one of its Passages, to link. */
function PassagePicker({ exclude, onPick }: PickerProps) {
  const [essays, setEssays] = useState<DocumentSummary[]>();
  const [essayId, setEssayId] = useState<number>();
  const [passages, setPassages] = useState<Passage[]>([]);
  const [error, setError] = useState<string>();

  useEffect(() => {
    if (essays === undefined) return;
    if (essayId === undefined) return setPassages([]);
    passageApi.ofDocument(essayId).then(setPassages, (e) => setError(toCoreError(e).message));
  }, [essays, essayId]);

  if (essays === undefined) {
    return (
      <button
        className="link-passage"
        onClick={() => documentApi.list("essay").then(setEssays, (e) => setError(toCoreError(e).message))}
      >
        Link a Passage…
      </button>
    );
  }

  const choices = passages.filter((p) => !p.orphaned && !exclude.includes(p.id));
  return (
    <div className="passage-picker">
      {error && <p className="error">{error}</p>}
      <select
        value={essayId ?? ""}
        aria-label="Essay"
        onChange={(e) => setEssayId(e.target.value ? Number(e.target.value) : undefined)}
      >
        <option value="">Choose an Essay</option>
        {essays.map((e) => (
          <option key={e.id} value={e.id}>
            {e.title || "Untitled"}
          </option>
        ))}
      </select>
      {essayId !== undefined && (
        <select
          value=""
          aria-label="Passage"
          onChange={(e) => {
            if (!e.target.value) return;
            onPick(e.target.value);
            setEssays(undefined);
            setEssayId(undefined);
          }}
        >
          <option value="">{choices.length === 0 ? "No Passages to link" : "Choose a Passage"}</option>
          {choices.map((p) => (
            <option key={p.id} value={p.id}>
              {p.lastKnownText.length > 80 ? `${p.lastKnownText.slice(0, 80)}…` : p.lastKnownText}
            </option>
          ))}
        </select>
      )}
      <button
        onClick={() => {
          setEssays(undefined);
          setEssayId(undefined);
        }}
      >
        Cancel
      </button>
    </div>
  );
}

export const designDecisionsSection: StyleSection = {
  id: "design-decisions",
  title: "Design Decisions",
  Component: DesignDecisionsSectionView,
};
