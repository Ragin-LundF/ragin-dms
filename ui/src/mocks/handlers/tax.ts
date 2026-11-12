import type {
    TaxAreaDto,
    TaxAreaGroupDto_Serialize,
    TaxAreaOptionDto,
    TaxAssignInputDto_Serialize,
    TaxDocumentChoiceDto_Serialize,
    TaxDocumentDto_Serialize,
    TaxItemChangeDto_Serialize,
    TaxItemDto_Serialize,
    TaxItemStateDto,
    TaxProgressDto,
    TaxYearViewDto_Serialize,
} from "../../ipc";
import {fileName, FIXTURE_DOCUMENTS} from "../fixture";
import type {Handler} from "./index";
import {updateMockSession} from "./shell";

const AREAS: [TaxAreaDto, string, string][] = [
  ["income", "Income", "Anlage N"],
  ["pensionInsurance", "Insurance and pension", "Vorsorgeaufwand"],
  ["householdServices", "Household services and craftsmen", "§ 35a EStG"],
  ["donations", "Donations", "Sonderausgaben"],
  ["healthCosts", "Health costs", "Außergewöhnliche Belastungen"],
  ["workExpenses", "Work-related expenses", "Werbungskosten"],
  ["capitalIncome", "Capital income", "Anlage KAP"],
  ["rental", "Rental income", "Anlage V"],
  ["other", "Other", "Other"],
];

export const TAX_AREA_LABELS = Object.fromEntries(AREAS.map(([area, label]) => [area, label])) as Record<TaxAreaDto, string>;

const AREA_OPTIONS: TaxAreaOptionDto[] = AREAS.map(([area, label, form]) => ({ area, label, form }));

/** The tax assignments of the synthetic documents, changed by the mocked commands. */
export const MOCK_TAX: Record<string, { year: number; area: TaxAreaDto; deductible: string | null }> = {
  "d-010": { year: 2025, area: "pensionInsurance", deductible: "412.30 EUR" },
  "d-011": { year: 2025, area: "pensionInsurance", deductible: null },
  "d-013": { year: 2025, area: "healthCosts", deductible: "212.75 EUR" },
  "d-015": { year: 2025, area: "householdServices", deductible: "684.50 EUR" },
};

type MockItem = { id: string; area: TaxAreaDto; label: string; correspondent: string | null; state: TaxItemStateDto; documentId: string | null; note: string | null; addedByUser: boolean };

const items: MockItem[] = [
  { id: "t-1", area: "income", label: "Lohnsteuerbescheinigung", correspondent: null, state: "missing", documentId: null, note: null, addedByUser: false },
  { id: "t-2", area: "pensionInsurance", label: "Health insurance certificate", correspondent: null, state: "expected", documentId: null, note: null, addedByUser: false },
  { id: "t-3", area: "pensionInsurance", label: "Safe Road Insurance", correspondent: "Safe Road Insurance", state: "found", documentId: "d-010", note: null, addedByUser: false },
  { id: "t-4", area: "householdServices", label: "Quick Garage", correspondent: "Quick Garage", state: "found", documentId: "d-015", note: null, addedByUser: false },
  { id: "t-5", area: "donations", label: "Sea Rescue", correspondent: "Sea Rescue", state: "missing", documentId: null, note: "Was there for 2023 and 2024.", addedByUser: false },
  { id: "t-6", area: "healthCosts", label: "Lakeside Clinic", correspondent: "Lakeside Clinic", state: "found", documentId: "d-013", note: null, addedByUser: false },
];
const opened = new Set<number>();
let nextId = 7;

function changed() {
  updateMockSession((state) => ({ ...state, revision: state.revision + 1 }));
}

function euros(text: string | null): number {
  return text ? Number(text.split(" ")[0]) : 0;
}

function display(amount: number): string {
  return `€ ${amount.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function itemDto(item: MockItem): TaxItemDto_Serialize {
  const document = FIXTURE_DOCUMENTS.find((candidate) => candidate.id === item.documentId);
  const tax = item.documentId ? MOCK_TAX[item.documentId] : undefined;
  return {
    id: item.id,
    label: item.label,
    correspondent: item.correspondent,
    state: item.state,
    documentId: item.documentId,
    detail: document ? document.title ?? fileName(document.path) : item.note,
    amount: tax?.deductible ? display(euros(tax.deductible)) : null,
    addedByUser: item.addedByUser,
  };
}

function view(year: number): TaxYearViewDto_Serialize {
  const shown = year === 2025 ? items : items.filter((item) => item.addedByUser);
  const counted = (list: MockItem[]) => ({ found: list.filter((item) => item.state === "found").length, total: list.filter((item) => item.state !== "notNeeded").length });
  const areas: TaxAreaGroupDto_Serialize[] = AREA_OPTIONS.flatMap((area) => {
    const ofArea = shown.filter((item) => item.area === area.area);
    if (ofArea.length === 0) return [];
    const status = ofArea.some((item) => item.state === "missing") ? "missing" : ofArea.some((item) => item.state === "expected") ? "expected" : "complete";
    return [{ area, ...counted(ofArea), status, items: ofArea.map(itemDto) }];
  });
  const sums = AREA_OPTIONS.flatMap((area) => {
    const total = Object.values(MOCK_TAX).filter((tax) => tax.year === year && tax.area === area.area).reduce((sum, tax) => sum + euros(tax.deductible), 0);
    return total > 0 ? [{ area, amounts: [display(total)] }] : [];
  });
  return { year, years: [2026, 2025, 2024], opened: opened.has(year), ...counted(shown), areas, sums, areaOptions: AREA_OPTIONS };
}

function item(id: string): MockItem {
  const found = items.find((candidate) => candidate.id === id);
  if (!found) throw { code: "invalid_input", message: "The document or file no longer exists" };
  return found;
}

/** Mocked commands of features::tax: the 2025 checklist of the synthetic library and its assignments. */
export const taxHandlers: Record<string, Handler> = {
  tax_year_view: (args): TaxYearViewDto_Serialize => view((args.year as number | null) ?? 2025),
  tax_open_year: (args) => {
    opened.add(args.year as number);
    changed();
    return true;
  },
  tax_update_item: (args) => {
    const change = args.change as TaxItemChangeDto_Serialize;
    switch (change.kind) {
      case "add":
        items.push({ id: `t-${nextId++}`, area: change.area, label: change.label.trim(), correspondent: change.correspondent, state: "missing", documentId: null, note: null, addedByUser: true });
        break;
      case "rename":
        item(change.itemId).label = change.label.trim();
        break;
      case "setNotNeeded":
        item(change.itemId).state = change.notNeeded ? "notNeeded" : "missing";
        break;
      case "linkDocument": {
        const target = item(change.itemId);
        Object.assign(target, { documentId: change.documentId, state: "found" });
        MOCK_TAX[change.documentId] = { year: args.year as number, area: target.area, deductible: MOCK_TAX[change.documentId]?.deductible ?? null };
        break;
      }
    }
    changed();
    return true;
  },
  tax_assign: (args) => {
    const input = args.input as TaxAssignInputDto_Serialize;
    for (const id of input.documentIds) {
      if (input.year === null || input.area === null) delete MOCK_TAX[id];
      else MOCK_TAX[id] = { year: input.year, area: input.area, deductible: input.keepDeductible ? MOCK_TAX[id]?.deductible ?? null : input.deductible?.trim() || null };
    }
    changed();
    return true;
  },
  tax_document: (args): TaxDocumentDto_Serialize => {
    const tax = MOCK_TAX[args.documentId as string];
    return {
      text: tax ? [String(tax.year), TAX_AREA_LABELS[tax.area], ...(tax.deductible ? [display(euros(tax.deductible))] : [])].join(" · ") : null,
      year: tax?.year ?? null,
      area: tax?.area ?? null,
      deductible: tax?.deductible ?? null,
      defaultYear: 2026,
      areaOptions: AREA_OPTIONS,
    };
  },
  tax_document_choices: (): TaxDocumentChoiceDto_Serialize[] =>
    FIXTURE_DOCUMENTS.map((document) => ({ id: document.id, name: document.title ?? fileName(document.path), correspondent: document.correspondent, date: document.documentDate })),
  tax_progress: (): TaxProgressDto => {
    const shown = view(2025);
    return { year: 2025, found: shown.found, total: shown.total, missing: items.filter((entry) => entry.state === "missing").length };
  },
};


