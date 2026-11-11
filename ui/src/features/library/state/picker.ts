import {createStore} from "../../../shared/store";

/**
 * The open "Move to…" picker, for documents and folders alike (Kotlin `FolderPicker`): its title, the selected folder
 * ("" is the library's top level), the unfolded folders, and `excluded`, a folder that cannot be the target together
 * with everything below it, such as the folder being moved. It only picks; `onPick` moves.
 */
export type PickerState = {
  title: string;
  confirmLabel: string;
  selected: string | null;
  expanded: string[];
  excluded: string | null;
  onPick: (folder: string) => void;
};

export const pickerStore = createStore<PickerState | null>(null);

export function isExcluded(state: Pick<PickerState, "excluded">, path: string): boolean {
  const folder = state.excluded;
  return folder !== null && (path === folder || path.startsWith(`${folder}/`));
}

export const picker = {
  open(title: string, onPick: (folder: string) => void, excluded: string | null = null) {
    pickerStore.set({ title, confirmLabel: "Move here", selected: null, expanded: [], excluded, onPick });
  },
  select(path: string) {
    pickerStore.set((state) => (state && !isExcluded(state, path) ? { ...state, selected: path } : state));
  },
  toggle(path: string) {
    pickerStore.set((state) =>
      state && { ...state, expanded: state.expanded.includes(path) ? state.expanded.filter((open) => open !== path) : [...state.expanded, path] },
    );
  },
  confirm() {
    const state = pickerStore.get();
    if (state?.selected == null) return;
    pickerStore.set(null);
    state.onPick(state.selected);
  },
  dismiss() {
    pickerStore.set(null);
  },
};
