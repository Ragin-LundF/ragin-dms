import { useState } from "react";

/**
 * A draft that starts from `source` and starts over when `source` changes (Kotlin `rememberSaveable(source)`): a
 * setting saved meanwhile replaces what was typed.
 */
export function useDraft<S, D>(source: S, toDraft: (source: S) => D): [D, (draft: D) => void] {
  const key = JSON.stringify(source);
  const [state, setState] = useState(() => ({ key, draft: toDraft(source) }));
  if (state.key !== key) {
    const fresh = { key, draft: toDraft(source) };
    setState(fresh);
    return [fresh.draft, (draft) => setState({ key, draft })];
  }
  return [state.draft, (draft) => setState({ key, draft })];
}
