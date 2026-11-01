import { useEffect, useRef } from "react";

/**
 * Steps that must finish before the shell leaves the open screen. The Essay
 * and Script screens add one that saves the editor, so a breadcrumb click,
 * ⌘[, or a sidebar row never loses an edit. A failed step does not stop
 * the shell: the editor shows its own save error.
 */
export function createLeaveGuards() {
  const guards = new Set<() => Promise<void>>();
  return {
    /** Returns the function that removes the step. */
    add(guard: () => Promise<void>): () => void {
      guards.add(guard);
      return () => void guards.delete(guard);
    },
    async run(): Promise<void> {
      await Promise.allSettled([...guards].map((guard) => guard()));
    },
  };
}

const leaveGuards = createLeaveGuards();

/** Runs every leave step. The shell awaits it before each navigation. */
export const runLeaveGuards = () => leaveGuards.run();

/** Runs `guard` before the shell leaves the screen, while the calling component is mounted. */
export function useLeaveGuard(guard: () => Promise<void>) {
  const latest = useRef(guard);
  latest.current = guard;
  useEffect(() => leaveGuards.add(() => latest.current()), []);
}
