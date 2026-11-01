import { useCallback, useEffect, useState, type DependencyList } from "react";
import { toCoreError, type CoreError } from "./coreError";

/**
 * Asks the core with `load`, then hands its answer to `onData` or its
 * refusal, as a CoreError, to `onError`. Returns the function that cancels:
 * after it, a late answer or refusal is dropped, so a screen that moved on
 * never shows stale data.
 */
export function startQuery<T>(
  load: () => Promise<T>,
  onData: (data: T) => void,
  onError: (error: CoreError) => void,
): () => void {
  let live = true;
  load().then(
    (data) => live && onData(data),
    (raw) => live && onError(toCoreError(raw)),
  );
  return () => {
    live = false;
  };
}

export interface CoreQuery<T> {
  /** The last answer; it stays while a new query runs. */
  data?: T;
  /** The last refusal; a later answer clears it. */
  error?: CoreError;
  /** Asks the core again. */
  reload(): void;
}

/**
 * Runs `load` each time a value in `deps` changes (and on `reload()`), with
 * the stale-answer cancel of {@link startQuery}. `load` must read only
 * values listed in `deps`.
 */
export function useCoreQuery<T>(load: () => Promise<T>, deps: DependencyList): CoreQuery<T> {
  const [state, setState] = useState<{ data?: T; error?: CoreError }>({});
  const [version, setVersion] = useState(0);
  useEffect(
    () =>
      startQuery(
        load,
        (data) => setState({ data }),
        (error) => setState((s) => ({ ...s, error })),
      ),
    // `deps` lists what `load` reads; `version` counts reloads.
    [...deps, version],
  );
  const reload = useCallback(() => setVersion((n) => n + 1), []);
  return { ...state, reload };
}
