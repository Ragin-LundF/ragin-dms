import {commands, describeError, type SplitPartDto_Serialize, type SplitProposalDto_Serialize, unwrap} from "../../ipc";
import {createStore} from "../../shared/store";

export type SplitPart = SplitPartDto_Serialize;

/**
 * The open "Split a stack" dialog: the AI's proposal (null while it reads), the cuts (a cut `n` starts a new document
 * at page `n`, pages from 0), the trash choice, or why there is no proposal.
 */
export type SplitDialogState = {
  documentId: string;
  name: string;
  proposal: SplitProposalDto_Serialize | null;
  cuts: number[];
  trashOriginal: boolean;
  problem: string | null;
};

export const splitDialog = createStore<SplitDialogState | null>(null);

/** Opens the dialog for the PDF `documentId` and asks the AI where each document starts. */
export async function openSplit(documentId: string, name: string) {
  splitDialog.set({ documentId, name, proposal: null, cuts: [], trashOriginal: false, problem: null });
  const still = (current: SplitDialogState | null) => current?.documentId === documentId;
  try {
    const proposal = await unwrap(commands.splitPropose(documentId));
    splitDialog.set((current) => (still(current) && current ? { ...current, proposal, cuts: cutsOf(proposal.parts) } : current));
  } catch (error) {
    splitDialog.set((current) => (still(current) && current ? { ...current, problem: describeError(error) } : current));
  }
}

/** The cuts the AI's parts stand for: every part but the first starts with one. */
export function cutsOf(parts: SplitPart[]): number[] {
  return parts.slice(1).map((part) => part.firstPage);
}

/** `cuts` with the cut before page `page` added or removed, in page order. */
export function toggledCut(cuts: number[], page: number): number[] {
  return cuts.includes(page) ? cuts.filter((cut) => cut !== page) : [...cuts, page].sort((a, b) => a - b);
}

/**
 * The documents the cuts make of `pageCount` pages. A part that starts where one of the AI's parts started keeps the
 * AI's title, category and confidence; any other part has none yet.
 */
export function partsOf(pageCount: number, cuts: number[], proposed: SplitPart[]): SplitPart[] {
  const starts = [0, ...cuts.filter((cut) => cut > 0 && cut < pageCount)].sort((a, b) => a - b);
  return starts.map((firstPage, index) => {
    const lastPage = (starts[index + 1] ?? pageCount) - 1;
    const known = proposed.find((part) => part.firstPage === firstPage);
    return known ? { ...known, lastPage } : { ...EMPTY_PART, firstPage, lastPage };
  });
}

const EMPTY_PART: SplitPart = {
  firstPage: 0, lastPage: 0, title: null, categoryId: null, newCategory: null, categoryLabel: null,
  correspondent: null, documentDate: null, confidence: null,
};
