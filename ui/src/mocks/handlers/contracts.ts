import type {
    ContractCardDto_Serialize,
    ContractEditorDto,
    ContractsOfDocumentDto_Serialize,
    ContractsStatusDto_Serialize,
    ContractsViewDto_Serialize,
    ContractTermsReviewDto_Serialize,
} from "../../ipc";
import type {Handler} from "./index";
import {mockOperation} from "./shell";

function form(id: string, name: string, correspondent: string, price: string, more: Partial<ContractEditorDto> = {}): ContractEditorDto {
  return {
    id, name, correspondent, kind: "telecom", price, period: "monthly", priceValidFrom: "", termEnd: "", renewsByMonths: "",
    noticeCount: "", noticeUnit: "months", cancellableAnyTime: false, note: "", documentIds: [], ...more,
  };
}

let cards: ContractCardDto_Serialize[] = [
  {
    id: "k-internet", name: "Internet 250", correspondent: "Wave Mobile", letter: "W", color: 0, monthly: "49.99 EUR", running: true,
    pills: [{ label: "Price ↑", tone: "warning" }, { label: "Cancel by 2026-11-30", tone: "danger" }],
    detail: {
      subtitle: "Wave Mobile · 1 document", monthly: "49.99 EUR", yearly: "599.88 EUR", notice: "1 month", cancelBy: "2026-11-30",
      termEnd: "2026-12-31", endedOn: null, increase: "+25 % since 2022", note: "Price rose in January.",
      history: [
        { validFrom: "2022-01-01", label: "39.99", cents: 3999, rose: false },
        { validFrom: "2024-01-01", label: "44.99", cents: 4499, rose: true },
        { validFrom: "2026-01-01", label: "49.99", cents: 4999, rose: true },
      ],
      documentIds: ["d-018"], documentsFilter: "Wave Mobile", documentsLabel: "Show 1 document",
      letterFacts: ["Contract: Internet 250", "End of the contract: 2026-12-31"], recipient: "Wave Mobile",
      editor: form("k-internet", "Internet 250", "Wave Mobile", "49.99 EUR", { termEnd: "2026-12-31", renewsByMonths: "12", noticeCount: "1" }),
    },
  },
  {
    id: "k-car", name: "Car insurance", correspondent: "Safe Road Insurance", letter: "S", color: 1, monthly: "40.60 EUR", running: true,
    pills: [{ label: "Next cancel-by 2027-09-30", tone: "neutral" }],
    detail: {
      subtitle: "Safe Road Insurance · 2 documents", monthly: "40.60 EUR", yearly: "487.20 EUR", notice: "3 months", cancelBy: "2027-09-30",
      termEnd: "2027-12-31", endedOn: null, increase: null, note: null, history: [],
      documentIds: ["d-010", "d-011"], documentsFilter: "Safe Road Insurance", documentsLabel: "Show 2 documents",
      letterFacts: ["Contract: Car insurance", "End of the contract: 2027-12-31"], recipient: "Safe Road Insurance",
      editor: form("k-car", "Car insurance", "Safe Road Insurance", "487.20 EUR", { kind: "insurance", period: "yearly", termEnd: "2027-12-31", renewsByMonths: "12", noticeCount: "3" }),
    },
  },
];

function update(id: string, change: (card: ContractCardDto_Serialize) => ContractCardDto_Serialize) {
  cards = cards.map((card) => (card.id === id ? change(card) : card));
}

/** Mocked commands of features::contracts, on two synthetic contracts. */
export const contractHandlers: Record<string, Handler> = {
  contracts_view: (): ContractsViewDto_Serialize => ({
    totals: [{ monthly: "90.59 EUR", yearly: "1087.08 EUR", change: "+ 5.00 EUR / month vs. last year", rising: true }],
    cards,
  }),
  contracts_status: (): ContractsStatusDto_Serialize | null =>
    cards.some((card) => card.running) ? { monthly: ["90.59 EUR"], increases: 1, increasesLabel: "1 price increase" } : null,
  contracts_of_document: ({ documentId }): ContractsOfDocumentDto_Serialize => ({
    contracts: cards.filter((card) => card.detail.documentIds.includes(documentId as string)).map(({ id, name }) => ({ id, name })),
    others: cards.filter((card) => card.running && !card.detail.documentIds.includes(documentId as string)).map(({ id, name }) => ({ id, name })),
    correspondent: null,
  }),
  contracts_new_form: ({ correspondent, documentIds }): ContractEditorDto =>
    form("", correspondent as string, correspondent as string, "", { kind: "other", documentIds: documentIds as string[] }),
  contracts_save: ({ form: saved }) => {
    const input = saved as ContractEditorDto;
    if (input.name.trim() === "") throw { code: "invalid_input", message: "Contract name must not be blank" };
    if (input.id) update(input.id, (card) => ({ ...card, name: input.name, detail: { ...card.detail, editor: input } }));
    mockOperation("Saving the contract", "library", 1, `Saved the contract ${input.name}`);
    return input.id || "k-internet";
  },
  contracts_end: ({ contractId, endedOn }) => {
    update(contractId as string, (card) => ({ ...card, running: endedOn === "", pills: endedOn ? [{ label: `Ended ${endedOn as string}`, tone: "neutral" }] : card.pills }));
    mockOperation("Ending the contract", "library", 1, endedOn ? "Ended the contract" : "The contract runs again");
    return null;
  },
  contracts_delete: ({ contractId }) => {
    cards = cards.filter((card) => card.id !== contractId);
    mockOperation("Deleting the contract", "library", 1, "Deleted the contract; its documents stay");
    return null;
  },
  contracts_attach: ({ detach }) => {
    mockOperation("Changing the contract's documents", "library", 1, detach ? "Taken off the contract" : "Added to the contract");
    return null;
  },
  contracts_extract_terms: (): ContractTermsReviewDto_Serialize => ({
    prices: [
      { label: "44.99 EUR / month from 2025-01-01", source: "Price change 2025.pdf", rose: true, point: { validFrom: "2025-01-01", price: "44.99 EUR", period: "monthly", documentId: "d-018" } },
    ],
    termsLabel: "Notice 1 month · ends 2026-12-31 · renews by 12 months",
    terms: { termEnd: "2026-12-31", renewsByMonths: "12", noticeCount: "1", noticeUnit: "months" },
    origin: "Read by AI (mock-model)",
  }),
  contracts_accept_price: () => {
    mockOperation("Saving the contract", "library", 1, "Added the price");
    return null;
  },
  contracts_accept_terms: () => {
    mockOperation("Saving the contract", "library", 1, "Took over the terms");
    return null;
  },
};
