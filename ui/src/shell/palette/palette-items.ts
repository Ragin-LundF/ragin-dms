import type {Lane, PaletteMatches_Serialize} from "../../ipc";
import type {LibrarySelection} from "../../features/library/navigation";
import {waitFor} from "../../shared/components";
import {PALETTE_ACTIONS, type PaletteActionId, paletteShortcut} from "./palette-actions";

/** One line of the palette and what choosing it does (Kotlin `PaletteItemModel`). */
export type PaletteItem =
  | { kind: "openDocument"; documentId: string; label: string; detail: string; hint: null }
  | { kind: "goTo"; selection: LibrarySelection; label: string; detail: string; hint: string | null }
  | { kind: "run"; action: PaletteActionId; label: string; detail: string | null; hint: string | null; disabledReason: string | null }
  | { kind: "ask"; question: string; label: string; detail: string; hint: string };

export type PaletteGroup = { title: string; items: PaletteItem[] };

/**
 * The groups for `query` (Kotlin `PaletteItems.of`): the documents and places Rust found, then the actions, which
 * say why they wait while their lane is busy. Without AI, no AI action and no question to the library.
 */
export function paletteGroups(query: string, matches: PaletteMatches_Serialize, busy: Partial<Record<Lane, string>>, aiEnabled: boolean, commandKey: string): PaletteGroup[] {
  const trimmed = query.trim();
  const wanted = trimmed.toLowerCase();
  const documents: PaletteItem[] = wanted ? matches.documents.map((document) => ({ kind: "openDocument", documentId: document.documentId, label: document.label, detail: document.detail, hint: null })) : [];
  const places: PaletteItem[] = wanted
    ? matches.places.map((place) => ({ kind: "goTo", selection: selectionOf(place.kind, place.value), label: place.label, detail: place.kind, hint: place.count === null ? null : String(place.count) }))
    : [];
  const actions: PaletteItem[] = (Object.keys(PALETTE_ACTIONS) as PaletteActionId[])
    .filter((id) => aiEnabled || PALETTE_ACTIONS[id].lane !== "ai")
    .filter((id) => !wanted || PALETTE_ACTIONS[id].label.toLowerCase().includes(wanted))
    .map((id) => {
      const lane = PALETTE_ACTIONS[id].lane;
      const running = lane ? busy[lane] : undefined;
      const disabledReason = running ? waitFor(running) : null;
      return { kind: "run", action: id, label: PALETTE_ACTIONS[id].label, detail: disabledReason, hint: paletteShortcut(id), disabledReason };
    });
  if (aiEnabled && trimmed) {
    actions.push({ kind: "ask", question: trimmed, label: `Ask the library: ${trimmed}`, detail: "Answers come from your documents, with sources", hint: `${commandKey}↵` });
  }
  return [
    { title: "Documents", items: documents },
    { title: "Go to", items: places },
    { title: "Actions", items: actions },
  ].filter((group) => group.items.length > 0);
}

function selectionOf(kind: string, value: string): LibrarySelection {
  switch (kind) {
    case "Tag": return { kind: "tag", tag: value };
    case "Category": return { kind: "category", categoryId: value };
    case "Smart folder": return { kind: "smartFolder", id: value };
    default: return { kind: "folder", path: value };
  }
}

/** Moves the selection by `steps`, wrapping around at both ends. */
export function moveSelection(selected: number, steps: number, count: number): number {
  if (count === 0) return 0;
  const current = Math.min(Math.max(selected, 0), count - 1);
  return (((current + steps) % count) + count) % count;
}
