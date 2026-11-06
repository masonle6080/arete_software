/**
 * Tells the Thesis dropdown and the Thesis Overview to reload. A screen
 * that adds, renames, or removes a Style, an Essay, or a Moodboard, moves a
 * Style's or a Thesis's status, or changes a Style's Cover calls
 * `announceThesisChange()`.
 * Screens in many areas do this, so a signal is simpler than a callback
 * passed through every screen. Announcing a change that is not in the
 * current Thesis only costs one extra reload.
 */
export function createThesisChanges() {
  const listeners = new Set<() => void>();
  return {
    announce() {
      for (const listener of [...listeners]) listener();
    },
    /** Returns the function that stops listening. */
    listen(listener: () => void): () => void {
      listeners.add(listener);
      return () => void listeners.delete(listener);
    },
  };
}

const thesisChanges = createThesisChanges();

export const announceThesisChange = () => thesisChanges.announce();
export const listenThesisChanges = thesisChanges.listen;
