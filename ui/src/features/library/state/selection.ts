import {createStore} from "../../../shared/store";

/** The documents ticked for a bulk action; `anchor` is where a Shift-click range starts (Kotlin `DocumentSelection`). */
export type Selection = { selected: ReadonlySet<string>; anchor: string | null };

export const NO_SELECTION: Selection = { selected: new Set(), anchor: null };

export const selectionStore = createStore<Selection>(NO_SELECTION);

/** Ticks or unticks `id` and makes it the anchor. */
export function toggled(state: Selection, id: string): Selection {
  const selected = new Set(state.selected);
  if (!selected.delete(id)) selected.add(id);
  return { selected, anchor: id };
}

/** Adds the documents from the anchor to `id` in shown `order`; without a shown anchor, only `id`, as the new anchor. */
export function extended(state: Selection, id: string, order: readonly string[]): Selection {
  const from = state.anchor === null ? -1 : order.indexOf(state.anchor);
  const to = order.indexOf(id);
  if (from < 0 || to < 0) return { selected: new Set([...state.selected, id]), anchor: id };
  const range = order.slice(Math.min(from, to), Math.max(from, to) + 1);
  return { selected: new Set([...state.selected, ...range]), anchor: state.anchor };
}

export function allOf(order: readonly string[]): Selection {
  return { selected: new Set(order), anchor: null };
}

/**
 * The ticked documents that are shown, in shown order. Ticks on documents a filter hides stay in memory and come
 * back with them, but bulk actions only act on the shown ones.
 */
export function chosen(state: Selection, order: readonly string[]): string[] {
  return order.filter((id) => state.selected.has(id));
}

export const selection = {
  toggle: (id: string) => selectionStore.set((state) => toggled(state, id)),
  extend: (id: string, order: readonly string[]) => selectionStore.set((state) => extended(state, id, order)),
  selectAll: (order: readonly string[]) => selectionStore.set(allOf(order)),
  clear: () => selectionStore.set(NO_SELECTION),
};
