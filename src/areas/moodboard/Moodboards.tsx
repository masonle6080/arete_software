import { useEffect, useState, type FormEvent } from "react";
import { toCoreError, type CoreError } from "../../shell/coreError";
import { announceThesisChange } from "../../shell/thesisChanges";
import { moodboardApi } from "./api";
import type { Moodboard, MoodboardOwner } from "./types";
import "./moodboard.css";
import { Button, EmptyText } from "../../ui";

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
      announceThesisChange();
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
      announceThesisChange();
    } catch (raw) {
      setError(toCoreError(raw));
    }
  }

  return (
    <div className="moodboards">
      {error && <p className="error">{error.message}</p>}
      {boards.length === 0 ? (
        <EmptyText>{single ? "No Moodboard yet." : "No Moodboards yet."}</EmptyText>
      ) : (
        <ul className="moodboard-list">
          {boards.map((b) => (
            <li key={b.id}>
              <strong>{b.title}</strong>
              <Button onClick={() => onOpen(b)}>Open</Button>
              {confirmDelete === b.id ? (
                <>
                  <Button onClick={() => remove(b)}>Confirm: delete this Moodboard</Button>
                  <Button onClick={() => setConfirmDelete(undefined)}>Cancel</Button>
                </>
              ) : (
                <Button onClick={() => setConfirmDelete(b.id)}>Delete</Button>
              )}
            </li>
          ))}
        </ul>
      )}
      {!(single && boards.length > 0) && (
        <form className="moodboard-create" onSubmit={create}>
          <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="New Moodboard title" />
          <Button type="submit" disabled={!title.trim()}>
            Add Moodboard
          </Button>
        </form>
      )}
    </div>
  );
}
