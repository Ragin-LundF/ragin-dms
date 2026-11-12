import type {
    EmergencyEditDto,
    EmergencyGroupDto,
    EmergencyItemDto_Serialize,
    EmergencyStateDto_Serialize
} from "../../ipc";
import {FIXTURE_DOCUMENTS} from "../fixture";
import type {Handler} from "./index";

const LABELS: Record<EmergencyGroupDto, string> = {
  contacts: "Contacts", insurance: "Insurance", accounts: "Accounts", contracts: "Contracts", personal: "Personal", other: "Other",
};
type MockItem = EmergencyItemDto_Serialize & { group: EmergencyGroupDto };

let nextId = 4;
const items: MockItem[] = [
  { id: "e-1", group: "contacts", kind: "contact", text: "Sample Tax Office · Tax adviser · 040 123456", included: true, changed: false, documentId: null },
  { id: "e-2", group: "insurance", kind: "document", text: `${FIXTURE_DOCUMENTS[0]?.path ?? "policy.pdf"}`, included: true, changed: true, documentId: FIXTURE_DOCUMENTS[0]?.id ?? null },
  { id: "e-3", group: "personal", kind: "note", text: "The paper folder is on the shelf in the study", included: false, changed: false, documentId: null },
];
let intro = "If something happens to me, this is where everything is.";

function state(): EmergencyStateDto_Serialize {
  const included = items.filter((item) => item.included);
  return {
    status: { title: "Last refreshed 2025-11-14", detail: "Next refresh on 2026-11-14 · 1 changed since then", refreshDue: true, changed: 1 },
    refreshMonths: 12,
    intro,
    groups: (Object.keys(LABELS) as EmergencyGroupDto[]).map((group) => ({
      group, label: LABELS[group], items: items.filter((item) => item.group === group),
    })),
    preview: {
      title: "Emergency folder",
      subtitle: "Alex Sample · as of 2026-11-11",
      intro,
      rows: included.map((item) => ({ area: LABELS[item.group], text: item.text })),
    },
    included: included.length,
    contracts: [{ id: "c-1", name: "Electricity", detail: "Sample Power" }],
    folders: ["", "Home", "Insurance"],
  };
}

function apply(edit: EmergencyEditDto): boolean {
  const add = (group: EmergencyGroupDto, kind: MockItem["kind"], text: string, documentId: string | null = null) =>
    items.push({ id: `e-${nextId++}`, group, kind, text, included: true, changed: false, documentId });
  switch (edit.kind) {
    case "addDocument": {
      const found = FIXTURE_DOCUMENTS.find((document) => document.id === edit.documentId);
      add(edit.group, "document", found?.path ?? edit.documentId, edit.documentId);
      return true;
    }
    case "addContract": add(edit.group, "contract", "Electricity · Sample Power"); return true;
    case "addContact": add(edit.group, "contact", [edit.name, edit.role, edit.phone, edit.email, edit.note].filter(Boolean).join(" · ")); return true;
    case "addNote": add(edit.group, "note", edit.text); return true;
    case "remove": items.splice(items.findIndex((item) => item.id === edit.itemId), 1); return true;
    case "setIncluded": items.forEach((item) => { if (item.id === edit.itemId) item.included = edit.included; }); return true;
    case "setIntro": intro = edit.intro; return true;
    case "setRefreshMonths": return true;
  }
}

/** Mocked commands of features::emergency: an in-memory folder with synthetic items. */
export const emergencyHandlers: Record<string, Handler> = {
  emergency_state: () => state(),
  emergency_edit: ({ edit }) => apply(edit as EmergencyEditDto),
  emergency_mark_refreshed: () => null,
  emergency_save_overview: () => true,
  emergency_export_zip: () => true,
  emergency_membership: ({ documentId, contractId }) => {
    const item = items.find((candidate) => candidate.documentId !== null && candidate.documentId === documentId);
    void contractId;
    return { itemId: item?.id ?? null, group: item ? LABELS[item.group] : null };
  },
  emergency_document_candidates: ({ query }) =>
    FIXTURE_DOCUMENTS.filter((document) => document.path.toLowerCase().includes(String(query).toLowerCase()))
      .slice(0, 20)
      .map((document) => ({ id: document.id, name: document.title ?? document.path, detail: document.path })),
};
