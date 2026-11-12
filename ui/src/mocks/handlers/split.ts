import type { PlainSearchResultDto_Serialize, SplitProposalDto_Serialize } from "../../ipc";
import { FIXTURE_DOCUMENTS } from "../fixture";
import type { Handler } from "./index";

const PAGES = 7;

/** Mocked commands of features::split and features::plain_search: a fixed proposal and a fixed translation. */
export const splitHandlers: Record<string, Handler> = {
  split_propose: (args): SplitProposalDto_Serialize => {
    const document = FIXTURE_DOCUMENTS.find((entry) => entry.id === args.documentId);
    if (!document || document.kind !== "pdf") throw { code: "invalid_input", message: "Only PDFs can be split" };
    const part = (firstPage: number, lastPage: number, title: string, categoryLabel: string, correspondent: string, confidence: number) => ({
      firstPage, lastPage, title, categoryId: null, newCategory: categoryLabel, categoryLabel: `${categoryLabel} (new)`, correspondent, documentDate: "2026-03-02", confidence,
    });
    return {
      documentId: document.id,
      name: document.path.split("/").pop() ?? document.path,
      pageCount: PAGES,
      model: "mock-model",
      parts: [part(0, 1, "Electricity bill March", "Bills", "Sample Energy", 96), part(2, 4, "Tax notice 2025", "Tax", "Sample Tax Office", 91), part(5, 6, "Pharmacy receipt", "Receipts", "Sample Pharmacy", 84)],
      note: null,
    };
  },
  split_apply: () => null,
  split_inbox_candidates: (): string[] => FIXTURE_DOCUMENTS.filter((document) => document.kind === "pdf" && document.path.startsWith("Inbox/")).map((document) => document.id),
  plain_search_translate: (args): PlainSearchResultDto_Serialize => {
    const sentence = String(args.sentence).toLowerCase();
    const year = /\b(20\d\d)\b/.exec(sentence)?.[1];
    return {
      filters: [
        ...(year ? [{ kind: "year" as const, year: Number(year), andEarlier: false }] : []),
        ...(/over \d+/.test(sentence) ? [{ kind: "field" as const, key: "amount", comparison: "greaterThan" as const, value: /over (\d+)/.exec(sentence)?.[1] ?? "0", valueType: "number" as const }] : []),
        ...(sentence.includes("unpaid") ? [{ kind: "payment" as const, status: "open" as const }] : []),
        ...(year && sentence.includes("tax") ? [{ kind: "tax" as const, year: Number(year), area: null }] : []),
      ],
      text: "",
      notUnderstood: sentence.includes("nordlicht") ? ["Nordlicht"] : [],
    };
  },
};
