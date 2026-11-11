import type {Lane} from "../../ipc";
import {shortcutLabel} from "../../shared/platform";

/**
 * The actions the palette offers: the ones of the menu bar, with the key of their shortcut, if any, and the lane
 * they run in; null for going to a screen, which never waits.
 */
export const PALETTE_ACTIONS = {
  scan: { label: "Scan now", key: "R", shift: false, lane: "library" },
  syncAll: { label: "Sync to all destinations", key: "S", shift: true, lane: "library" },
  mergeAll: { label: "Merge all folders", key: "M", shift: true, lane: "library" },
  retryFailed: { label: "Retry failed operations", key: null, shift: false, lane: "library" },
  detectDates: { label: "Detect missing document dates", key: null, shift: false, lane: "library" },
  addFiles: { label: "Add files…", key: null, shift: false, lane: "library" },
  tagUntagged: { label: "Tag untagged documents with AI", key: null, shift: false, lane: "ai" },
  classifyUncategorized: { label: "Classify uncategorized documents with AI", key: null, shift: false, lane: "ai" },
  convertToRealPdf: { label: "Convert the open PDF to a real PDF", key: null, shift: false, lane: "ai" },
  openDashboard: { label: "Go to Dashboard", key: "1", shift: false, lane: null },
  openTasks: { label: "Go to Open tasks", key: "2", shift: false, lane: null },
  openLibrary: { label: "Go to Library", key: "3", shift: false, lane: null },
  openSettings: { label: "Go to Settings", key: ",", shift: false, lane: null },
} satisfies Record<string, { label: string; key: string | null; shift: boolean; lane: Lane | null }>;

export type PaletteActionId = keyof typeof PALETTE_ACTIONS;

/** The shortcut of an action as menus show it ("⌘⇧S"); null without one. */
export function paletteShortcut(action: PaletteActionId): string | null {
  const { key, shift } = PALETTE_ACTIONS[action];
  return key ? shortcutLabel(key, shift) : null;
}
