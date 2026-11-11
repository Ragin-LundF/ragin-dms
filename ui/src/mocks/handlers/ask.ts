import type {AskEntryDto_Serialize, AskStateDto_Serialize} from "../../ipc";
import {fileName, FIXTURE_DOCUMENTS, FIXTURE_SMART_FOLDERS} from "../fixture";
import type {Handler} from "./index";
import {mockRun} from "./restructure";
import {mockSession} from "./shell";

const ask: { entries: AskEntryDto_Serialize[]; indexed: boolean } = { entries: [], indexed: false };

function answer(question: string): AskEntryDto_Serialize {
  const words = question.toLowerCase().split(/[^\p{L}\p{N}]+/u).filter((word) => word.length >= 4);
  const found = FIXTURE_DOCUMENTS.filter((document) => {
    const text = `${document.path} ${document.title ?? ""} ${document.correspondent ?? ""}`.toLowerCase();
    return words.some((word) => text.includes(word));
  }).slice(0, 3);
  if (found.length === 0) return { question, answer: null, citations: [], note: "Nothing in your documents fits this question.", asking: false };
  const citations = found.map((document, index) => ({ number: index + 1, documentId: document.id, title: document.title ?? fileName(document.path) }));
  const text = `According to ${citations.map((citation) => `[${citation.number}]`).join(" and ")}, the answer is in ${citations[0]?.title ?? ""}: the synthetic library has no more detail than that.`;
  return { question, answer: text, citations, note: ask.indexed ? null : "Searched by words only; update the index to search by meaning", asking: false };
}

/** Mocked commands of features::ask. */
export const askHandlers: Record<string, Handler> = {
  ask_state: (): AskStateDto_Serialize => ({ entries: ask.entries, scopes: ["Whole library", ...FIXTURE_SMART_FOLDERS.map((search) => `Smart folder: ${search.name}`)] }),
  ask_library: (args) => {
    const question = (args.question as string).trim();
    if (question === "") return null;
    const position = ask.entries.length;
    ask.entries = [...ask.entries, { question, answer: null, citations: [], note: null, asking: true }];
    try {
      mockRun("Asking the library", "ai", 4, true, () => {
        ask.entries = ask.entries.map((entry, index) => (index === position ? answer(question) : entry));
        return null;
      });
    } catch {
      const running = mockSession.state.operations.find((operation) => operation.lane === "ai")?.label ?? "";
      ask.entries = ask.entries.map((entry, index) => (index === position ? { ...entry, asking: false, note: `Wait until "${running}" finishes` } : entry));
    }
    return null;
  },
  update_ask_index: () => {
    mockRun("Updating the index", "ai", 10, true, () => {
      ask.indexed = true;
      return { text: `The index is up to date; ${FIXTURE_DOCUMENTS.length} documents were read anew`, isError: false };
    });
    return null;
  },
};
