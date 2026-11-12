import type {
    CaseChoiceDto,
    CaseDetailDto_Serialize as CaseDetailDto,
    CaseRowDto_Serialize as CaseRowDto,
    CasesFilterDto,
    CasesNewDto,
    CasesOfDocumentDto,
    CasesReviewDto_Serialize as CasesReviewDto,
    CaseStatusDto,
    CasesViewDto_Serialize,
    CaseTimelineItemDto_Serialize as CaseTimelineItemDto,
    LinkedDocument_Serialize as LinkedDocument,
} from "../../ipc";
import {FIXTURE_DOCUMENTS} from "../fixture";
import type {Handler} from "./index";

/** One mocked case: entries are document ids with their direction, follow-ups by id. */
type MockCase = {
  id: string; title: string; correspondent: string | null; status: CaseStatusDto; waitingDays: number;
  entries: { documentId: string; direction: "received" | "sent"; date: string }[];
  followUps: { id: string; text: string; date: string; done: boolean; ai: boolean }[];
  notes: { date: string; text: string }[];
};

const TODAY = "2026-11-11";
const first = FIXTURE_DOCUMENTS[0];
let cases: MockCase[] = [
  {
    id: "case-tax", title: "Tax office asks for receipts", correspondent: "Tax office", status: "open", waitingDays: 0,
    entries: first ? [{ documentId: first.id, direction: "received", date: "2026-11-04" }] : [],
    followUps: [{ id: "f-1", text: "Send the plumber invoices", date: "2026-11-10", done: false, ai: true }],
    notes: [],
  },
  { id: "case-objection", title: "Objection to the service charges", correspondent: "Sample Housing", status: "waiting", waitingDays: 24, entries: [], followUps: [], notes: [] },
  { id: "case-mobile", title: "Cancel old mobile contract", correspondent: "Mobile North", status: "done", waitingDays: 0, entries: [], followUps: [], notes: [] },
];
let next = 1;

const name = (id: string) => FIXTURE_DOCUMENTS.find((document) => document.id === id)?.title ?? id;
const open = (item: MockCase) => item.followUps.filter((followUp) => !followUp.done).sort((a, b) => a.date.localeCompare(b.date));
const find = (id: unknown) => {
  const found = cases.find((item) => item.id === id);
  if (!found) throw { code: "invalid_input", message: "The case no longer exists" };
  return found;
};

function pill(item: MockCase) {
  const attention = item.status !== "done" && open(item).some((followUp) => followUp.date <= TODAY);
  const label = item.status === "open" ? (attention ? "Action needed" : "Open") : item.status === "waiting" ? "Waiting" : "Done";
  return { status: item.status, label, attention };
}

function row(item: MockCase): CaseRowDto {
  const followUp = open(item)[0];
  const nextStep = item.status === "done" ? "Done on 2026-10-02" : followUp ? `${followUp.text} by ${followUp.date}`
    : item.status === "waiting" ? `Waiting for an answer for ${item.waitingDays} days` : "No next step";
  return { id: item.id, title: item.title, correspondent: item.correspondent, pill: pill(item), nextStep };
}

function detail(item: MockCase): CaseDetailDto {
  const step = (kind: CaseTimelineItemDto["kind"], date: string, word: string, title: string): CaseTimelineItemDto =>
    ({ kind, date, marker: `${date} · ${word}`, title, detail: null, documentId: null, followUpId: null, done: false, due: false });
  const timeline = [
    ...item.entries.map((entry) => ({ ...step(entry.direction, entry.date, entry.direction, name(entry.documentId)), documentId: entry.documentId })),
    ...item.followUps.map((followUp) => ({
      ...step("followUp", followUp.date, "follow-up", followUp.text), followUpId: followUp.id, done: followUp.done,
      due: !followUp.done && followUp.date <= TODAY, detail: followUp.done ? "Done" : followUp.ai ? "Proposed by the AI from the letter" : null,
    })),
    ...item.notes.map((note) => step("note", note.date, "note", note.text)),
  ].sort((a, b) => a.date.localeCompare(b.date));
  const aiActions = open(item).filter((followUp) => followUp.ai).map((followUp) => followUp.text);
  const letter = item.entries[0];
  return {
    id: item.id, title: item.title, pill: pill(item), timeline,
    summary: [item.correspondent, `${item.entries.length} ${item.entries.length === 1 ? "document" : "documents"}`].filter(Boolean).join(" · "),
    requests: letter && aiActions.length ? [{ documentId: letter.documentId, documentName: name(letter.documentId), actions: aiActions }] : [],
    reminder: { caseId: item.id, documentIds: item.entries.map((entry) => entry.documentId), recipient: item.correspondent },
  };
}

const choice = (item: MockCase): CaseChoiceDto => ({ id: item.id, title: item.title, status: item.status });
const candidate = (id: string): LinkedDocument => ({ documentId: id, name: name(id), path: FIXTURE_DOCUMENTS.find((document) => document.id === id)?.path ?? id, label: null });
const followUpId = () => `f-mock-${next++}`;

/** Mocked commands of features::cases, on an in-memory list of invented cases. */
export const caseHandlers: Record<string, Handler> = {
  cases_view: ({ filter, selected }): CasesViewDto_Serialize => {
    const shown = cases.filter((item) => (filter as CasesFilterDto) === "done" ? item.status === "done" : item.status !== "done");
    const chosen = shown.find((item) => item.id === selected) ?? shown[0];
    return {
      activeCount: cases.filter((item) => item.status !== "done").length,
      doneCount: cases.filter((item) => item.status === "done").length,
      cases: shown.map(row),
      detail: chosen ? detail(chosen) : null,
    };
  },
  cases_of_document: ({ documentId }): CasesOfDocumentDto => ({
    cases: cases.filter((item) => item.entries.some((entry) => entry.documentId === documentId)).map(choice),
    candidates: cases.filter((item) => item.status !== "done" && !item.entries.some((entry) => entry.documentId === documentId)).map(choice),
    hasLinks: false,
  }),
  cases_document_candidates: ({ caseId, query }) => {
    const item = find(caseId);
    const wanted = String(query).toLowerCase();
    return FIXTURE_DOCUMENTS.filter((document) => !item.entries.some((entry) => entry.documentId === document.id))
      .filter((document) => `${document.path} ${document.title ?? ""} ${document.correspondent ?? ""}`.toLowerCase().includes(wanted))
      .slice(0, 30).map((document) => candidate(document.id));
  },
  cases_review: (): CasesReviewDto => ({
    requested: [{ text: "Send the receipts for the craftsmen", date: "2026-11-30" }],
    matches: FIXTURE_DOCUMENTS.slice(1, 3).map((document) => candidate(document.id)),
    openCase: choice(cases[0] as MockCase),
    preChecked: false,
    newTitle: "Request for receipts",
  }),
  cases_create: ({ input }) => {
    const created = input as CasesNewDto;
    const id = `case-mock-${next++}`;
    cases = [...cases, {
      id, title: created.title, correspondent: created.correspondent ?? null, status: "open", waitingDays: 0, followUps: [], notes: [],
      entries: created.documentIds.map((documentId) => ({ documentId, direction: "received", date: TODAY })),
    }];
    return id;
  },
  cases_create_from_links: ({ documentId }) => caseHandlers.cases_create?.({ input: { title: name(String(documentId)), correspondent: null, documentIds: [documentId] } }),
  cases_add_document: ({ caseId, documentId, direction }) => {
    const item = find(caseId);
    item.entries = [...item.entries.filter((entry) => entry.documentId !== documentId), { documentId: String(documentId), direction: direction as "received" | "sent", date: TODAY }];
    if (direction === "received" && item.status === "waiting") item.status = "open";
    return null;
  },
  cases_remove_document: ({ caseId, documentId }) => {
    const item = find(caseId);
    item.entries = item.entries.filter((entry) => entry.documentId !== documentId);
    return null;
  },
  cases_set_status: ({ caseId, status }) => {
    find(caseId).status = status as CaseStatusDto;
    return null;
  },
  cases_add_note: ({ caseId, text }) => {
    find(caseId).notes.push({ date: TODAY, text: String(text) });
    return null;
  },
  cases_add_follow_up: ({ caseId, followUp }) => {
    const { text, date } = followUp as { text: string; date: string };
    if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) throw { code: "invalid_input", message: "The date must be YYYY-MM-DD" };
    find(caseId).followUps.push({ id: followUpId(), text, date, done: false, ai: false });
    return null;
  },
  cases_complete_follow_up: ({ caseId, followUpId: id }) => {
    const followUp = find(caseId).followUps.find((item) => item.id === id);
    if (followUp) followUp.done = true;
    return null;
  },
  cases_snooze_follow_up: ({ caseId, followUpId: id }) => {
    const followUp = find(caseId).followUps.find((item) => item.id === id);
    if (followUp) followUp.date = "2026-11-18";
    return null;
  },
  cases_delete: ({ caseId }) => {
    cases = cases.filter((item) => item.id !== caseId);
    return null;
  },
  cases_file_letter: () => null,
};
