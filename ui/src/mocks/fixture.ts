/**
 * The synthetic library every mocked command answers from, so all screens show the same documents. Invented data
 * only. Features map it to their own DTOs in their handler file and may add what they need here.
 */

export type FixtureDocument = {
  id: string;
  path: string;
  title: string | null;
  kind: "pdf" | "image" | "office" | "other";
  size: number;
  addedAt: string;
  documentDate: string | null;
  correspondent: string | null;
  categoryId: string | null;
  tags: string[];
  /** The AI suggested metadata that waits for review. */
  suggestion: boolean;
  note: string | null;
};

export const FIXTURE_FOLDERS = { primary: "f-primary", sync: "f-sync" } as const;

export const FIXTURE_CATEGORIES = [
  { id: "c-finance", name: "Finance", parentId: null },
  { id: "c-tax", name: "Tax", parentId: "c-finance" },
  { id: "c-bank", name: "Bank statements", parentId: "c-finance" },
  { id: "c-home", name: "Home", parentId: null },
  { id: "c-insurance", name: "Insurance", parentId: null },
  { id: "c-health", name: "Health", parentId: null },
];

export const FIXTURE_TAGS = ["invoice", "contract", "2025", "2026", "warranty", "urgent", "car"];

export const FIXTURE_DIRECTORIES = ["Finance", "Finance/Tax", "Finance/Tax/2025", "Home", "Insurance", "Health", "Inbox"];

const day = (offset: number) => new Date(Date.now() - offset * 86_400_000).toISOString();
const date = (offset: number) => day(offset).slice(0, 10);

function doc(index: number, path: string, fields: Partial<FixtureDocument>): FixtureDocument {
  const extension = path.split(".").pop() ?? "";
  const kind = extension === "pdf" ? "pdf" : ["png", "jpg", "jpeg"].includes(extension) ? "image" : ["docx", "xlsx"].includes(extension) ? "office" : "other";
  return {
    id: `d-${String(index).padStart(3, "0")}`,
    path,
    title: null,
    kind,
    size: 40_000 + index * 37_517,
    addedAt: day(index * 3),
    documentDate: date(index * 5 + 2),
    correspondent: null,
    categoryId: null,
    tags: [],
    suggestion: false,
    note: null,
    ...fields,
  };
}

export const FIXTURE_DOCUMENTS: FixtureDocument[] = [
  doc(1, "Finance/Tax/2025/Income tax return 2025.pdf", { title: "Income tax return 2025", correspondent: "Tax office", categoryId: "c-tax", tags: ["2025"] }),
  doc(2, "Finance/Tax/2025/Tax assessment.pdf", { title: "Tax assessment 2025", correspondent: "Tax office", categoryId: "c-tax", tags: ["2025", "urgent"] }),
  doc(3, "Finance/Bank statement 2026-01.pdf", { correspondent: "Northbank", categoryId: "c-bank", tags: ["2026"] }),
  doc(4, "Finance/Bank statement 2026-02.pdf", { correspondent: "Northbank", categoryId: "c-bank", tags: ["2026"] }),
  doc(5, "Finance/Bank statement 2026-03.pdf", { correspondent: "Northbank", categoryId: "c-bank", tags: ["2026"] }),
  doc(6, "Home/Rental contract.pdf", { title: "Rental contract", correspondent: "Maple Housing", categoryId: "c-home", tags: ["contract"], note: "Notice period three months." }),
  doc(7, "Home/Electricity invoice March.pdf", { correspondent: "City Power", categoryId: "c-home", tags: ["invoice", "2026"] }),
  doc(8, "Home/Dishwasher warranty.jpg", { title: "Dishwasher warranty", categoryId: "c-home", tags: ["warranty"] }),
  doc(9, "Home/Floor plan.png", { categoryId: "c-home" }),
  doc(10, "Insurance/Car insurance policy.pdf", { correspondent: "Safe Road Insurance", categoryId: "c-insurance", tags: ["contract", "car"] }),
  doc(11, "Insurance/Household insurance.pdf", { correspondent: "Safe Road Insurance", categoryId: "c-insurance", tags: ["contract"] }),
  doc(12, "Health/Vaccination record.pdf", { categoryId: "c-health" }),
  doc(13, "Health/Lab results.pdf", { correspondent: "Lakeside Clinic", categoryId: "c-health", suggestion: true }),
  doc(14, "Budget 2026.xlsx", { tags: ["2026"] }),
  doc(15, "Car repair invoice.pdf", { correspondent: "Quick Garage", tags: ["invoice", "car"], suggestion: true }),
  doc(16, "Scan 2026-03-02.pdf", { documentDate: null, suggestion: true }),
  doc(17, "Inbox/scan-2026-03-08.pdf", { documentDate: null, addedAt: day(0) }),
  doc(18, "Inbox/Phone bill.pdf", { correspondent: "Wave Mobile", addedAt: day(1), suggestion: true }),
  doc(19, "Inbox/receipt.jpg", { documentDate: null, addedAt: day(1) }),
  doc(20, "Notes.txt", { documentDate: null }),
];

/** The documents of a folder and below it ("" is the root). */
export function documentsBelow(folder: string): FixtureDocument[] {
  return FIXTURE_DOCUMENTS.filter((document) => folder === "" || document.path.startsWith(`${folder}/`));
}

export function fileName(path: string): string {
  return path.split("/").pop() ?? path;
}

// ---- document area (quick look, details, review): custom fields, links, retention and suggestion details ----

export type FixtureFieldDefinition = { key: string; label: string; type: "text" | "number" | "money" | "date" | "boolean"; role: "dueDate" | "cancelBy" | "contractEnd" | "warrantyEnd" | null };

/** The standard fields; their values are stored as the formatted text the quick look shows. */
export const FIXTURE_FIELD_DEFINITIONS: FixtureFieldDefinition[] = [
  { key: "amount", label: "Amount", type: "money", role: null },
  { key: "due", label: "Due date", type: "date", role: "dueDate" },
  { key: "cancel-by", label: "Cancel by", type: "date", role: "cancelBy" },
  { key: "contract-end", label: "Contract end", type: "date", role: "contractEnd" },
  { key: "warranty-until", label: "Warranty until", type: "date", role: "warrantyEnd" },
];

export const FIXTURE_DEFAULT_CURRENCY = "EUR";

export const FIXTURE_FIELD_VALUES: Record<string, Record<string, string>> = {
  "d-007": { amount: "84.20 EUR", due: date(-5) },
  "d-015": { amount: "312.00 EUR", due: date(-30) },
  "d-018": { amount: "39.99 EUR", due: date(2) },
  "d-010": { "cancel-by": date(-40), "contract-end": date(-70) },
  "d-008": { "warranty-until": date(-400) },
};

export const FIXTURE_LINKS: { first: string; second: string; label: string | null }[] = [
  { first: "d-001", second: "d-002", label: "assessment" },
  { first: "d-010", second: "d-011", label: null },
];

/** Opt-in retention: a keep-until date on the document, or years on a category, or marked as done. */
export const FIXTURE_RETENTION: Record<string, { keepUntil: string | null; settled: boolean }> = {
  "d-001": { keepUntil: "2036-12-31", settled: false },
  "d-012": { keepUntil: null, settled: true },
};
export const FIXTURE_CATEGORY_RETENTION_YEARS: Record<string, number> = { "c-tax": 10 };

export type FixtureSuggestion = {
  model: string;
  confidence: number | null;
  title: string | null;
  /** An existing category's path ("Finance / Tax") or a new one. */
  category: { path: string; isNew: boolean } | null;
  correspondent: string | null;
  documentDate: string | null;
  fileName: string | null;
  targetFolder: { path: string; isNew: boolean; reason: string | null } | null;
  tags: string[];
  fields: Record<string, string>;
  ruleNotes: Partial<Record<"category" | "tags" | "fields" | "targetFolder", string>>;
};

/** What the AI suggested for the documents with `suggestion: true`. */
export const FIXTURE_SUGGESTIONS: Record<string, FixtureSuggestion> = {
  "d-013": {
    model: "gemma-3-12b", confidence: 0.91, title: "Blood test results", category: { path: "Health", isNew: false }, correspondent: "Lakeside Clinic",
    documentDate: date(40), fileName: null, targetFolder: null, tags: ["lab"], fields: {}, ruleNotes: {},
  },
  "d-015": {
    model: "gemma-3-12b", confidence: 0.84, title: "Car repair invoice", category: { path: "Finance / Car", isNew: true }, correspondent: "Quick Garage",
    documentDate: date(46), fileName: "Quick Garage repair.pdf", targetFolder: { path: "Finance", isNew: false, reason: "Invoices are filed under Finance" },
    tags: ["car", "invoice"], fields: { amount: "312.00 EUR" }, ruleNotes: {},
  },
  "d-016": {
    model: "rules", confidence: null, title: null, category: { path: "Finance / Bank statements", isNew: false }, correspondent: "Northbank",
    documentDate: null, fileName: null, targetFolder: null, tags: ["2026"], fields: {}, ruleNotes: { category: "Northbank statements", tags: "Northbank statements" },
  },
  "d-018": {
    model: "gemma-3-12b", confidence: 0.77, title: "Phone bill March", category: { path: "Home", isNew: false }, correspondent: "Wave Mobile",
    documentDate: date(7), fileName: null, targetFolder: { path: "Home/Phone", isNew: true, reason: "Bills of Wave Mobile" },
    tags: ["invoice"], fields: { amount: "39.99 EUR", due: date(-2) }, ruleNotes: {},
  },
};

// ---- restructure, ask and pdf: smart folders as scopes, applied restructures, page counts ----

export const FIXTURE_SMART_FOLDERS = [
  { id: "s-contracts", name: "Contracts" },
  { id: "s-car", name: "Car" },
];

/** Applied restructures, newest first (record names are UTC times). */
export const FIXTURE_RESTRUCTURE_HISTORY = ["20260302-091512", "20260114-180344"];

/** Pages of the PDFs the page editor opens; others have 3. */
export const FIXTURE_PAGE_COUNTS: Record<string, number> = { "d-001": 6, "d-006": 4, "d-016": 2, "d-017": 1 };
