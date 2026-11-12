import {type CasesFilterDto, type CasesReviewChoiceDto, type CasesReviewDto, commands, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {createStore} from "../../shared/store";
import {goTo} from "../../shell/screens";

/** The cases screen's list and the chosen case; kept while the screen is left, like the library's selection. */
export const casesScreen = createStore<{ filter: CasesFilterDto; selected: string | null }>({ filter: "active", selected: null });

/** "Draft reminder" and other letters written for a case: the letter dialog with the case's documents. */
export type CaseLetter = { caseId: string | null; documentIds: string[]; recipient: string | null };
export const caseLetterStore = createStore<CaseLetter | null>(null);

/** "Add document": the case that gets a document from the picker, and the search typed. */
export const documentPickStore = createStore<{ caseId: string; query: string } | null>(null);

/** What the open review also does with cases on "Apply checked values"; null does nothing. */
export const reviewCaseChoice = createStore<CasesReviewChoiceDto | null>(null);

/** Shows the cases screen with `caseId` chosen, in the list that holds it. */
export function openCase(caseId: string, done = false) {
  casesScreen.set({ filter: done ? "done" : "active", selected: caseId });
  goTo("cases");
}

export function draftCaseLetter(letter: CaseLetter) {
  caseLetterStore.set(letter);
}

/** Runs a case change and reports a refusal; resolves to whether it was done. */
export function change(command: Promise<unknown>): Promise<boolean> {
  return command.then(() => true, (error: unknown) => {
    reportError(error);
    return false;
  });
}

/** Starts a case and shows it. */
export async function startCase(title: string, correspondent: string | null, documentIds: string[]) {
  try {
    openCase(await unwrap(commands.casesCreate({ title, correspondent, documentIds })));
  } catch (error) {
    reportError(error);
  }
}

/** What the review's case part answers: the checked requests and where they go, or joining the open case. */
export type ReviewCaseState = { checked: boolean[]; target: "new" | "existing"; join: boolean };

/**
 * The case choice of a review: checked requests (only with AI on) start a case or go to the correspondent's open
 * case; without requests the document may join that case; otherwise nothing happens with cases.
 */
export function reviewChoiceOf(review: CasesReviewDto, aiEnabled: boolean, state: ReviewCaseState): CasesReviewChoiceDto | null {
  const followUps = aiEnabled ? review.requested.filter((_, index) => state.checked[index] ?? true) : [];
  const open = review.openCase;
  if (followUps.length > 0) {
    return open && state.target === "existing"
      ? { kind: "addToCase", caseId: open.id, followUps }
      : { kind: "newCase", title: review.newTitle, followUps };
  }
  return open && state.join ? { kind: "addToCase", caseId: open.id, followUps: [] } : null;
}
