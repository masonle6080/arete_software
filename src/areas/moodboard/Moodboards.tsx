import { useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { moodboardApi } from "./api";
import type { Moodboard, MoodboardOwner } from "./types";
import "./moodboard.css";

interface Props {
  owner: MoodboardOwner;
  /** The owner has at most one Moodboard (a Style): hide "Add" once it has one. */
  single?: boolean;
  /** Opens one Moodboard's canvas on its own screen (see `MoodboardScreen`). */
  onOpen(board: Moodboard): void;
}

/**
 * The Moodboards of one owner: create, open, and delete them. Reusable for
 * any owner kind (a Thesis or a Style). The canvas opens on its own screen.
 */
export function Moodboards({ owner, single = false, onOpen }: Props) {
  const [boards, setBoards] = useState<Moodboard[]>([]);
  const [title, setTitle] = useState("");
  const [error, setError] = useState<CoreError>();
  /** The Moodboard whose Delete button waits for a confirm click. */
  const [confirmDelete, setConfirmDelete] = useState<number>();

  const { kind, id } = owner;
  useEffect(() => {
    moodboardApi.listMoodboards({ kind, id }).then(setBoards, (raw) => setError(toCoreError(raw)));
  }, [kind, id]);

  async function create(e: FormEvent) {
    e.preventDefault();
    setError(undefined);
    try {
      const board = await moodboardApi.createMoodboard({ kind, id }, title);
      setTitle("");
      onOpen(board);
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  async function remove(board: Moodboard) {
    setError(undefined);
    setConfirmDelete(undefined);
    try {
      await moodboardApi.deleteMoodboard(board.id);
      setBoards((list) => list.filter((b) => b.id !== board.id));
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <div className="moodboards">
      {error && <p className="error">{error.message}</p>}
      {boards.length === 0 ? (
        <p className="empty">{single ? "No Moodboard yet." : "No Moodboards yet."}</p>
      ) : (
        <ul className="moodboard-list">
          {boards.map((b) => (
            <li key={b.id}>
              <strong>{b.title}</strong>
              <button onClick={() => onOpen(b)}>Open</button>
              {confirmDelete === b.id ? (
                <>
                  <button onClick={() => remove(b)}>Confirm: delete this Moodboard</button>
                  <button onClick={() => setConfirmDelete(undefined)}>Cancel</button>
                </>
              ) : (
                <button onClick={() => setConfirmDelete(b.id)}>Delete</button>
              )}
            </li>
          ))}
        </ul>
      )}
      {!(single && boards.length > 0) && (
        <form className="moodboard-create" onSubmit={create}>
          <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="New Moodboard title" />
          <button type="submit" disabled={!title.trim()}>
            Add Moodboard
          </button>
        </form>
      )}
    </div>
  );
}
