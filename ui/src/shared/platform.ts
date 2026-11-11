import {open, save} from "@tauri-apps/plugin-dialog";
import {commands} from "../ipc";
import {setNumberLocale} from "./format/sizes";

/** The operating system the app runs on, and its conventions (Kotlin `HostOs`). */
export const isMac = navigator.userAgent.includes("Mac");

/** How the command modifier is written: "⌘" on macOS, "Ctrl+" elsewhere. */
export const commandKey = isMac ? "⌘" : "Ctrl+";

/** A shortcut as menus show it, e.g. `shortcutLabel("S", true)` is "⌘⇧S". */
export function shortcutLabel(key: string, shift = false): string {
  return commandKey + (shift ? "⇧" : "") + key;
}

/** Whether the platform's command modifier (⌘ or Ctrl) is held. */
export function commandHeld(event: { metaKey: boolean; ctrlKey: boolean }): boolean {
  return isMac ? event.metaKey : event.ctrlKey;
}

let fileManager = isMac ? "Finder" : "file manager";
void commands.fileManagerName().then((name) => (fileManager = name), () => undefined);
void commands.numberLocale().then(setNumberLocale, () => undefined);

/** "Finder", "Explorer" or "file manager", for labels such as "Show in Finder". */
export function fileManagerName(): string {
  return fileManager;
}

/** Asks the user for a folder; its absolute path, or null if the user cancelled. */
export async function chooseFolder(title: string): Promise<string | null> {
  const chosen = await open({ title, directory: true, multiple: false });
  return typeof chosen === "string" ? chosen : null;
}

/** Asks the user for files to add; their absolute paths, empty if the user cancelled. */
export async function chooseFiles(title: string): Promise<string[]> {
  const chosen = await open({ title, multiple: true, directory: false });
  if (!chosen) return [];
  return Array.isArray(chosen) ? chosen : [chosen];
}

/** Asks where to save a file; null if the user cancelled. */
export async function chooseSaveLocation(title: string, defaultPath?: string): Promise<string | null> {
  return save({ title, defaultPath });
}
