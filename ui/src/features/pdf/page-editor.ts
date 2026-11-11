import { commands, unwrap, type PageItemDto } from "../../ipc";
import { reportError } from "../../session/session";
import { createStore } from "../../shared/store";

/** The open page editor of one PDF: its pages in their new order (null while counting) and the trash option. */
export type PageEditorState = { documentId: string; name: string; pages: PageItemDto[] | null; trashOriginal: boolean };

export const pageEditor = createStore<PageEditorState | null>(null);

const FULL_TURN = 360;

export async function openPageEditor(documentId: string, name: string) {
  pageEditor.set({ documentId, name, pages: null, trashOriginal: false });
  try {
    const count = await unwrap(commands.pdfPageCount(documentId));
    const pages = Array.from({ length: count }, (_, source) => ({ source, rotation: 0, deleted: false, splitAfter: false }));
    pageEditor.set((current) => (current?.documentId === documentId ? { ...current, pages } : current));
  } catch (error) {
    pageEditor.set(null);
    reportError(error);
  }
}

export function changePage(pages: PageItemDto[], position: number, change: (page: PageItemDto) => PageItemDto): PageItemDto[] {
  return pages.map((page, index) => (index === position ? change(page) : page));
}

/** Adds `degrees` to the page's turn, kept within 0..359. */
export function turned(page: PageItemDto, degrees: number): PageItemDto {
  return { ...page, rotation: (((page.rotation + degrees) % FULL_TURN) + FULL_TURN) % FULL_TURN };
}

/** Moves the page at `position` one place to the left (−1) or the right (+1); at an end nothing moves. */
export function moved(pages: PageItemDto[], position: number, by: number): PageItemDto[] {
  const target = position + by;
  if (target < 0 || target >= pages.length) return pages;
  const next = [...pages];
  next.splice(target, 0, ...next.splice(position, 1));
  return next;
}

/**
 * What the dialog shows about the edit: pages kept, whether the file gets a new version (pages turned, removed or
 * moved) and how many splits are marked (on any page but the last, not removed).
 */
export function editSummary(pages: PageItemDto[]): { kept: number; edited: boolean; splits: number } {
  return {
    kept: pages.filter((page) => !page.deleted).length,
    edited: pages.some((page, index) => page.rotation !== 0 || page.deleted || page.source !== index),
    splits: pages.slice(0, -1).filter((page) => page.splitAfter && !page.deleted).length,
  };
}
