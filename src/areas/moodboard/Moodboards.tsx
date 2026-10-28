import { lazy, Suspense, useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { moodboardApi } from "./api";
import type { Moodboard, MoodboardOwner } from "./types";
import "./moodboard.css";

// Excalidraw is large; load it only when a Moodboard opens.
const MoodboardCanvas = lazy(() => import("./MoodboardCanvas"));

interface Props {
  owner: MoodboardOwner;
}

/**
 * The Moodboards of one owner: create, pick, and edit them. Reusable for any
 * owner kind (a Thesis now, a Style later).
 */
export function Moodboards({ owner }: Props) {
  const [boards, setBoards] = useState<Moodboard[]>([]);
  const [openId, setOpenId] = useState<number>();
  const [title, setTitle] = useState("");
  const [error, setError] = useState<CoreError>();

  const { kind, id } = owner;
  useEffect(() => {
    moodboardApi.listMoodboards({ kind, id }).then(
      (list) => {
        setBoards(list);
        setOpenId((current) => current ?? list[0]?.id);
      },
      (raw) => setError(toCoreError(raw)),
    );
  }, [kind, id]);

  async function create(e: FormEvent) {
    e.preventDefault();
    setError(undefined);
    try {
      const board = await moodboardApi.createMoodboard({ kind, id }, title);
      setBoards((list) => [...list, board]);
      setOpenId(board.id);
      setTitle("");
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <div className="moodboards">
      <div className="moodboard-bar">
        {boards.map((b) => (
          <button key={b.id} className={b.id === openId ? "active" : undefined} onClick={() => setOpenId(b.id)}>
            {b.title}
          </button>
        ))}
        <form onSubmit={create}>
          <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="New Moodboard title" />
          <button type="submit" disabled={!title.trim()}>
            Add Moodboard
          </button>
        </form>
      </div>
      {error && <p className="error">{error.message}</p>}
      {boards.length === 0 && <p className="empty">No Moodboards yet.</p>}
      {openId !== undefined && (
        <Suspense fallback={<p className="empty">Loading canvas…</p>}>
          <MoodboardCanvas key={openId} moodboardId={openId} />
        </Suspense>
      )}
    </div>
  );
}
