import {commands, describeError, type QuickFact, type Review_Serialize, type ReviewRowDraft, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {createStore} from "../../shared/store";
import {answer, ask, type Chat, clear, newChat, stop} from "./assistant/chat-state";

/**
 * The one quick look of the library (Kotlin `QuickLookController`): the open document and the order next/previous
 * walk. The library, the folder view, the keys and the palette all open the same.
 */
export const quickLook = createStore<{ documentId: string | null; order: string[] }>({ documentId: null, order: [] });

/** The value the quick look edits in place; it belongs to one document, so showing another one ends it unsaved. */
export const quickLookEditing = createStore<{ documentId: string; fact: QuickFact } | null>(null);

export function openQuickLook(documentId: string, order: string[]) {
  quickLook.set({ documentId, order });
}

export function closeQuickLook() {
  quickLook.set((current) => ({ ...current, documentId: null }));
}

/** ↓ / ↑: the next or previous document in view order; stops at the ends and when the shown one left the view. */
export function stepQuickLook(by: 1 | -1) {
  const { documentId, order } = quickLook.get();
  if (documentId === null) return;
  const index = order.indexOf(documentId);
  const target = index < 0 ? undefined : order[index + by];
  if (target !== undefined) quickLook.set({ documentId: target, order });
}

export function editFact(documentId: string, fact: QuickFact) {
  quickLookEditing.set({ documentId, fact });
}

export function endEdit() {
  quickLookEditing.set(null);
}

/** Saves a value typed in place; a refused value keeps the edit open so it can be fixed. */
export function saveFact(documentId: string, fact: QuickFact, typed: string) {
  unwrap(commands.saveQuickFact(documentId, fact, typed)).then((saved) => saved && endEdit(), reportError);
}

/** The open review: the suggestion's rows as the user edits them, and why the last approval was refused. */
export type OpenReview = { review: Review_Serialize; rows: ReviewRowDraft[]; problem: string | null };
export const reviewStore = createStore<OpenReview | null>(null);

export function closeReview() {
  reviewStore.set(null);
}

/** Opens the review of the AI suggestion for `documentId`, with `problem` when "Accept all" was refused. */
export function openReview(documentId: string, problem: string | null = null) {
  unwrap(commands.suggestionReview(documentId)).then((review) => {
    if (!review) return;
    const rows = review.rows.map((row) => ({ key: row.key, accepted: row.accepted, value: row.value }));
    reviewStore.set({ review, rows, problem });
  }, reportError);
}

/** The AI's summary of one document; `token` drops an answer that arrives after the dialog closed. */
export type Summary = { documentId: string; fileName: string; working: boolean; text: string | null; model: string | null; problem: string | null; token: number };
export const summaryStore = createStore<Summary | null>(null);
let summaries = 0;

export function openSummary(documentId: string, fileName: string) {
  const token = ++summaries;
  summaryStore.set({ documentId, fileName, working: true, text: null, model: null, problem: null, token });
  const settle = (result: Pick<Summary, "text" | "model" | "problem">) => {
    if (summaryStore.get()?.token === token) summaryStore.set((current) => current && { ...current, ...result, working: false });
  };
  unwrap(commands.summarizeDocument(documentId)).then(settle, (error) => settle({ text: null, model: null, problem: describeError(error) }));
}

export function closeSummary() {
  summaryStore.set(null);
}

/** The conversation about one document; opening it again for the same document keeps the conversation. */
export const chatStore = createStore<Chat | null>(null);

export function openChat(documentId: string, fileName: string) {
  if (chatStore.get()?.documentId !== documentId) chatStore.set(newChat(documentId, fileName));
}

export function typeQuestion(input: string) {
  const chat = chatStore.get();
  if (chat) setChat({ ...chat, input });
}

export function sendQuestion(question?: string) {
  const chat = chatStore.get();
  const asked = chat && ask(chat, question ?? chat.input);
  if (!asked) return;
  setChat(asked);
  const settle = (result: { text: string | null; problem: string | null }) => {
    const current = chatStore.get();
    if (current?.documentId === asked.documentId) setChat(answer(current, asked.asked, result));
  };
  unwrap(commands.askAboutDocument(asked.documentId, asked.turns)).then(settle, (error) => settle({ text: null, problem: describeError(error) }));
}

export function stopAnswer() {
  const chat = chatStore.get();
  if (chat) setChat(stop(chat));
}

export function startOver() {
  const chat = chatStore.get();
  if (chat) setChat(clear(chat));
}

/** Close: stops the AI and discards the conversation. */
export function closeChat() {
  chatStore.set(null);
}

function setChat(chat: Chat) {
  chatStore.set(chat);
}

/** The open "Link to…" picker of one document: what the user searches for and the label of the new link. */
export const linkPickStore = createStore<{ documentId: string; query: string; label: string } | null>(null);

export function closeLinkPick() {
  linkPickStore.set(null);
}
