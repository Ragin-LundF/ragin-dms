import {createStore} from "../../shared/store";

/** The dialog of the bills features that is open over any screen: a bill's payment code, or a request letter. */
export type BillsOverlay =
  | { kind: "paymentCode"; documentId: string }
  | { kind: "requestLetter"; recipient: string; documentIds: string[]; subjectFacts: string[] };

export const billsOverlay = createStore<BillsOverlay | null>(null);

export function openPaymentCode(documentId: string) {
  billsOverlay.set({ kind: "paymentCode", documentId });
}

export function openRequestLetter(recipient: string, documentIds: string[], subjectFacts: string[]) {
  billsOverlay.set({ kind: "requestLetter", recipient, documentIds, subjectFacts });
}
