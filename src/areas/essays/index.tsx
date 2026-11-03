import { useCallback, useEffect, useRef, useState } from "react";
import type { Area, AreaProps } from "../../shell/area";
import { useLeaveGuard } from "../../shell/leaveGuards";
import { steps, type Step } from "../../shell/navPath";
import { toCoreError } from "../../shell/coreError";
import { announceTreeChange } from "../../shell/treeChanges";
import { DocumentEditor, type DocumentEditorHandle, type DocumentSave } from "../../document/DocumentEditor";
import { OrphanedPassages } from "../passage/OrphanedPassages";
import { PassageView } from "../passage/PassageView";
import { documentApi, type DocumentSummary, type StoredDocument } from "./api";
import { EssayExport } from "./EssayExport";
import { EssayTheses } from "./EssayTheses";
import "./essays.css";
import { Button, EmptyText, icons } from "../../ui";

function EssayList({ onOpen }: { onOpen(id: number, title: string): void }) {
  const [essays, setEssays] = useState<DocumentSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    documentApi.list("essay").then(setEssays, (e) => setError(toCoreError(e).message));
  }, []);

  const create = async () => {
    try {
      const essay = await documentApi.create("essay", "");
      onOpen(essay.id, essay.title);
    } catch (e) {
      setError(toCoreError(e).message);
    }
  };

  return (
    <section>
      <header className="essay-list-header">
        <h1>Essays</h1>
        <Button onClick={create}>New essay</Button>
      </header>
      {error && <p className="error">{error}</p>}
      {essays?.length === 0 && <EmptyText>No essays yet.</EmptyText>}
      <ul className="essay-list">
        {essays?.map((e) => (
          <li key={e.id}>
            <Button onClick={() => onOpen(e.id, e.title)}>
              <span>{e.title || "Untitled"}</span>
              <time dateTime={e.updatedAt}>{new Date(e.updatedAt).toLocaleString()}</time>
            </Button>
          </li>
        ))}
      </ul>
    </section>
  );
}

interface EssayScreenProps {
  id: number;
  /** When set, the Essay opens with this Passage selected and in view. */
  focusPassageId?: string;
  /** The Passage open over the editor, if any. The shell owns it: it is one level of the navigation path. */
  passageId?: string;
  /** Opens one Passage of this Essay over the editor. */
  onOpenPassage(passageId: string): void;
  /** Receives the title when the Essay loads and after each save that changes it, for the breadcrumb. */
  onTitle?(title: string): void;
  /** Opens what a header chip names, such as a Thesis the Essay supports. */
  onOpen(step: Step): void;
}

/**
 * One Essay in the editor, with its Thesis links and orphaned Passages.
 * Other areas can open it too. A Passage opens in place over the editor; the
 * editor stays mounted, so undo still works after coming back. The shell
 * saves the editor before it leaves this screen.
 */
export function EssayScreen({ id, focusPassageId, passageId, onOpenPassage, onTitle, onOpen }: EssayScreenProps) {
  const [essay, setEssay] = useState<StoredDocument | null>(null);
  const [error, setError] = useState<string | null>(null);
  const editor = useRef<DocumentEditorHandle>(null);
  const [saves, setSaves] = useState(0);
  /** The last saved title, to tell the Thesis Overview and the breadcrumb when it changes. */
  const savedTitle = useRef<string>();
  const onTitleRef = useRef(onTitle);
  onTitleRef.current = onTitle;

  useLeaveGuard(async () => editor.current?.flush());

  useEffect(() => {
    documentApi.open(id).then(
      (doc) => {
        savedTitle.current = doc.title;
        setEssay(doc);
        onTitleRef.current?.(doc.title);
      },
      (e) => setError(toCoreError(e).message),
    );
  }, [id]);

  const save = useCallback(
    async ({ title, markdown, passages }: DocumentSave) => {
      await documentApi.save(id, { title, markdown }, passages);
      setSaves((n) => n + 1);
      if (title !== savedTitle.current) {
        savedTitle.current = title;
        onTitleRef.current?.(title);
        announceTreeChange();
      }
    },
    [id],
  );

  const openPassage = useCallback(
    async (pid: string) => {
      // The Passage must be saved before the core knows it.
      await editor.current?.flush();
      onOpenPassage(pid);
    },
    [onOpenPassage],
  );

  return (
    <section>
      {passageId && <PassageView key={passageId} id={passageId} />}
      <div hidden={passageId !== undefined}>
        {error && <p className="error">{error}</p>}
        {essay && <EssayTheses essayId={essay.id} version={saves} onOpen={onOpen} />}
        {essay && <EssayExport essayId={essay.id} flush={async () => editor.current?.flush()} />}
        {essay && <OrphanedPassages documentId={essay.id} version={saves} onOpen={openPassage} />}
        {essay && (
          <DocumentEditor
            ref={editor}
            key={essay.id}
            title={essay.title}
            markdown={essay.markdown}
            onSave={save}
            onOpenPassage={openPassage}
            focusPassage={focusPassageId}
          />
        )}
      </div>
    </section>
  );
}

/** The Shared Essays list. An Essay opens one level down in the navigation path (`Essays › On Ash`). */
function EssaysArea({ onOpen }: AreaProps) {
  return <EssayList onOpen={(id, title) => onOpen(steps.essay(id, title))} />;
}

export const essaysArea: Area = { id: "essays", label: "Essays", icon: icons.essays, Screen: EssaysArea };
