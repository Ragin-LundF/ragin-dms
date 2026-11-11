import { commands, unwrap, type LibraryLayoutDto } from "../../../ipc";
import { reportError } from "../../../session/session";
import { createStore } from "../../../shared/store";

/** The library's two panels the user resizes and folds away (Kotlin `LayoutPanel`). */
export type Panel = "sidebar" | "folderTree";

const PANELS: Record<Panel, { min: number; max: number; initial: number; width: keyof LibraryLayoutDto; folded: keyof LibraryLayoutDto }> = {
  sidebar: { min: 150, max: 360, initial: 200, width: "sidebarWidth", folded: "sidebarFolded" },
  folderTree: { min: 160, max: 480, initial: 240, width: "folderTreeWidth", folded: "folderTreeHidden" },
};

/** How far below the narrowest width a drag must end to fold the panel. */
const FOLD_DISTANCE = 50;

export const DEFAULT_LAYOUT: LibraryLayoutDto = { sidebarWidth: 200, sidebarFolded: false, folderTreeWidth: 240, folderTreeHidden: false };

/** The stored layout and the widths being dragged now; a drag is saved only when it ends (Kotlin `PanelWidth`). */
export type PanelState = { stored: LibraryLayoutDto; dragged: Partial<Record<Panel, number>> };
export const panelStore = createStore<PanelState>({ stored: DEFAULT_LAYOUT, dragged: {} });

export function clamp(panel: Panel, width: number): number {
  const { min, max } = PANELS[panel];
  return Math.min(max, Math.max(min, Math.round(width)));
}

/** How wide `panel` is now, also while its edge is dragged, and whether it is folded away. */
export function panelOf(state: PanelState, panel: Panel): { width: number; folded: boolean } {
  const dragged = state.dragged[panel];
  const width = dragged === undefined ? (state.stored[PANELS[panel].width] as number) : clamp(panel, dragged);
  return { width, folded: state.stored[PANELS[panel].folded] as boolean };
}

/** The layout after a drag of `panel`'s edge ended at `width`: folded (keeping its width) when far below the narrowest. */
export function afterDrag(layout: LibraryLayoutDto, panel: Panel, width: number): LibraryLayoutDto {
  const { min } = PANELS[panel];
  if (width < min - FOLD_DISTANCE) return { ...layout, [PANELS[panel].folded]: true };
  return { ...layout, [PANELS[panel].width]: clamp(panel, width) };
}

function store(layout: LibraryLayoutDto) {
  panelStore.set((state) => ({ ...state, stored: layout }));
  unwrap(commands.librarySetLayout(layout)).catch(ignoreUnwired);
}

/** Until the backend keeps the layout, it only lives for this run. */
function ignoreUnwired(error: unknown) {
  if ((error as { code?: string }).code !== "not_wired") reportError(error);
}

export const panels = {
  load() {
    unwrap(commands.libraryLayout())
      .then((stored) => panelStore.set((state) => ({ ...state, stored })))
      .catch(ignoreUnwired);
  },
  resize(panel: Panel, delta: number) {
    panelStore.set((state) => {
      const start = state.dragged[panel] ?? (state.stored[PANELS[panel].width] as number);
      return { ...state, dragged: { ...state.dragged, [panel]: start + delta } };
    });
  },
  endResize(panel: Panel) {
    const { stored, dragged } = panelStore.get();
    const width = dragged[panel];
    if (width === undefined) return;
    panelStore.set((state) => ({ ...state, dragged: { ...state.dragged, [panel]: undefined } }));
    store(afterDrag(stored, panel, width));
  },
  resetWidth(panel: Panel) {
    store({ ...panelStore.get().stored, [PANELS[panel].width]: PANELS[panel].initial });
  },
  toggleFolded(panel: Panel) {
    const stored = panelStore.get().stored;
    store({ ...stored, [PANELS[panel].folded]: !stored[PANELS[panel].folded] });
  },
};
