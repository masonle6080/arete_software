/**
 * Tells the Thesis dropdown and the Thesis Overview to reload. A screen
 * that adds, renames, or removes a Style, an Essay, or a Moodboard, moves a
 * Style's or a Thesis's status, or changes a Style's Cover calls
 * `announceTreeChange()`.
 * Screens in many areas do this, so a signal is simpler than a callback
 * passed through every screen. Announcing a change that is not in the
 * current Thesis only costs one extra reload.
 */
export function createTreeChanges() {
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

const treeChanges = createTreeChanges();

export const announceTreeChange = () => treeChanges.announce();
export const listenTreeChanges = treeChanges.listen;
