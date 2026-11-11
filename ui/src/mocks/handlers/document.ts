import {emit} from "@tauri-apps/api/event";
import type {
    ChatMessage,
    FieldFact_Serialize as FieldFact,
    QuickLook_Serialize,
    Review_Serialize,
    ReviewRow_Serialize,
    ReviewRowDraft,
    TypedField
} from "../../ipc";
import {relativeDay} from "../../shared/format/dates";
import {
    fileName,
    FIXTURE_CATEGORIES,
    FIXTURE_CATEGORY_RETENTION_YEARS,
    FIXTURE_DEFAULT_CURRENCY,
    FIXTURE_DOCUMENTS,
    FIXTURE_FIELD_DEFINITIONS,
    FIXTURE_FIELD_VALUES,
    FIXTURE_LINKS,
    FIXTURE_RETENTION,
    FIXTURE_SUGGESTIONS,
    FIXTURE_TAGS,
    type FixtureDocument,
    type FixtureFieldDefinition,
    type FixtureSuggestion,
} from "../fixture";
import type {Handler} from "./index";
import {mockOperation, mockSession, updateMockSession} from "./shell";

type Category = { id: string; name: string; parentId: string | null };
const categories = FIXTURE_CATEGORIES as Category[];

/** Mocked commands of features::document: the quick look, its details, the review and the assistant, on the fixture. */
export const documentHandlers: Record<string, Handler> = {
  quick_look: ({ documentId }) => quickLookOf(find(documentId as string)),
  save_quick_fact: ({ documentId, fact, typed }) => saveFact(document(documentId), fact as string, typed as string),
  assign_document_category: ({ documentId, categoryId }) => change(() => (document(documentId).categoryId = categoryId as string | null)),
  remove_document_tag: ({ documentId, tag }) =>
    change(() => {
      const target = document(documentId);
      target.tags = target.tags.filter((existing) => existing.toLowerCase() !== (tag as string).toLowerCase());
    }),
  open_document_file: () => null,
  show_document_in_file_manager: () => null,
  rename_document: ({ documentId, name }) => moveTo(document(documentId), folderOf(document(documentId).path), (name as string).trim()),
  move_document_to_folder: ({ documentId, folder }) => {
    const target = document(documentId);
    return moveTo(target, (folder as string).trim().replace(/^[/\\]+|[/\\]+$/g, ""), fileName(target.path));
  },
  trash_question: ({ documentId }) => ({
    title: "Move to trash",
    message: "The file goes to the trash in every library folder and leaves the catalog.",
    affected: ["Documents", "Cloud"].map((name) => `${name}: ${document(documentId).path}`),
    confirmLabel: "Move to trash",
  }),
  move_document_to_trash: ({ documentId }) => {
    const target = document(documentId);
    FIXTURE_DOCUMENTS.splice(FIXTURE_DOCUMENTS.indexOf(target), 1);
    return operation(`Moving ${target.path} to the trash`, `Moved ${target.path} to the trash`);
  },
  classify_document_with_ai: ({ documentId }) => suggest(document(documentId), "Classifying"),
  tag_document_with_ai: ({ documentId }) => suggest(document(documentId), "Tagging"),
  suggestion_review: ({ documentId }) => reviewOf(document(documentId)),
  approve_review: ({ documentId, rows }) => approve(document(documentId), rows as ReviewRowDraft[]),
  accept_suggestion: ({ documentId }) => {
    const review = reviewOf(document(documentId));
    return review ? approve(document(documentId), review.rows.map(({ key, accepted, value }) => ({ key, accepted, value }))) : null;
  },
  reject_review: ({ documentId }) => reject(document(documentId)),
  dismiss_suggestion: ({ documentId }) => reject(document(documentId)),
  add_standard_fields: () => operation("Adding the standard fields", "Added the standard fields"),
  save_document_fields: ({ documentId, fields }) => saveFields(document(documentId), fields as TypedField[]),
  save_document_note: ({ documentId, note }) => {
    const target = document(documentId);
    const trimmed = (note as string).trimEnd();
    if (trimmed === (target.note ?? "")) return null;
    return change(() => (target.note = trimmed.trim() === "" ? null : trimmed));
  },
  save_keep_until: ({ documentId, typed }) => {
    const raw = (typed as string).trim();
    if (raw !== "" && !isIsoDate(raw)) return reportError("Write the date as YYYY-MM-DD, such as 2030-12-31");
    const current = FIXTURE_RETENTION[documentId as string] ?? { keepUntil: null, settled: false };
    return change(() => (FIXTURE_RETENTION[documentId as string] = { ...current, keepUntil: raw === "" ? null : raw }));
  },
  save_category_years: ({ categoryId, typed }) => {
    const raw = (typed as string).trim();
    const years = Number(raw);
    if (raw !== "" && !(Number.isInteger(years) && years >= 1 && years <= 100)) return reportError("Write a number of years from 1 to 100");
    return change(() => {
      if (raw === "") delete FIXTURE_CATEGORY_RETENTION_YEARS[categoryId as string];
      else FIXTURE_CATEGORY_RETENTION_YEARS[categoryId as string] = years;
    });
  },
  create_rule_from_document: ({ documentId }) => {
    const target = document(documentId);
    if (!target.correspondent || (!target.categoryId && target.tags.length === 0)) return reportError("Give the document a correspondent and a category or tags first");
    return operation(`Creating Rule for ${target.correspondent}`, `Created Rule for ${target.correspondent}`);
  },
  link_candidates_of: ({ documentId, query }) => {
    const wanted = (query as string).trim().toLowerCase();
    return FIXTURE_DOCUMENTS.filter((other) => other.id !== documentId)
      .filter((other) => wanted === "" || [other.path, other.title, other.correspondent].filter(Boolean).join(" ").toLowerCase().includes(wanted))
      .sort((first, second) => second.addedAt.localeCompare(first.addedAt))
      .slice(0, 30)
      .map(candidate);
  },
  link_documents: ({ documentId, otherId, label }) =>
    change(() => {
      const trimmed = (label as string).trim();
      const existing = FIXTURE_LINKS.findIndex((link) => samePair(link, documentId as string, otherId as string));
      if (existing >= 0) FIXTURE_LINKS.splice(existing, 1);
      FIXTURE_LINKS.push({ first: documentId as string, second: otherId as string, label: trimmed === "" ? null : trimmed });
    }),
  unlink_documents: ({ documentId, otherId }) =>
    change(() => {
      const existing = FIXTURE_LINKS.findIndex((link) => samePair(link, documentId as string, otherId as string));
      if (existing >= 0) FIXTURE_LINKS.splice(existing, 1);
    }),
  related_documents_of: ({ documentId }) => {
    const shown = find(documentId as string);
    if (!shown?.correspondent) return [];
    return FIXTURE_DOCUMENTS.filter((other) => other.id !== shown.id && other.correspondent === shown.correspondent).slice(0, 3).map(candidate);
  },
  summarize_document: async ({ documentId }) => {
    const target = document(documentId);
    await pause(1200);
    return {
      text: `${target.title ?? fileName(target.path)} comes from ${target.correspondent ?? "an unknown sender"}.\n\nIt lists the details of the matter and asks for no action within the next weeks.`,
      model: "gemma-3-12b",
      problem: null,
    };
  },
  ask_about_document: async ({ conversation }) => {
    const turns = conversation as ChatMessage[];
    await pause(900);
    const question = turns[turns.length - 1]?.text ?? "";
    return { text: `The document answers "${question}" on its first page: nothing is due before the end of the month.`, problem: null };
  },
};

function find(id: string): FixtureDocument | undefined {
  return FIXTURE_DOCUMENTS.find((candidate) => candidate.id === id);
}

function document(id: unknown): FixtureDocument {
  const found = find(id as string);
  if (!found) throw { code: "invalid_input", message: "The document is no longer in the catalog" };
  return found;
}

/** A catalog change "Saving the catalog": applied at once, then the screens reload. */
function change(apply: () => void): null {
  apply();
  updateMockSession((state) => ({ ...state, revision: state.revision + 1 }));
  return null;
}

/** A longer operation: the change is already applied, the status bar shows it running, and `done` is reported. */
function operation(label: string, done: string): null {
  mockOperation(label, "library", 3, done);
  return null;
}

function reportError(text: string): null {
  void emit("message-event", { message: { text, isError: true } });
  return null;
}

function pause(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

const today = () => new Date().toISOString().slice(0, 10);
const isIsoDate = (text: string) => /^\d{4}-\d{2}-\d{2}$/.test(text) && !Number.isNaN(Date.parse(`${text}T00:00:00Z`));
const folderOf = (path: string) => (path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "");
const byName = (first: string, second: string) => first.toLowerCase().localeCompare(second.toLowerCase());
const samePair = (link: { first: string; second: string }, first: string, second: string) =>
  (link.first === first && link.second === second) || (link.first === second && link.second === first);

function candidate(target: FixtureDocument) {
  return { documentId: target.id, name: target.title ?? fileName(target.path), path: target.path, label: null };
}

function categoryPath(id: string | null): string | null {
  const category = categories.find((candidate) => candidate.id === id);
  if (!category) return null;
  const parent = categoryPath(category.parentId);
  return parent ? `${parent} / ${category.name}` : category.name;
}

function categoryChoices(parentId: string | null = null, depth = 0): QuickLook_Serialize["categories"] {
  return categories
    .filter((category) => category.parentId === parentId)
    .sort((first, second) => byName(first.name, second.name))
    .flatMap((category) => [{ id: category.id, name: category.name, depth }, ...categoryChoices(category.id, depth + 1)]);
}

const LEAD_DAYS = { dueDate: 7, cancelBy: 30, contractEnd: 30, warrantyEnd: 30 };

function fieldFact(target: FixtureDocument, definition: FixtureFieldDefinition | undefined): FieldFact | null {
  if (!definition) return null;
  const value = FIXTURE_FIELD_VALUES[target.id]?.[definition.key] ?? null;
  if (value === null || definition.type !== "date" || definition.role === null) return { label: definition.label, value, chip: null, urgent: false };
  const days = Math.round((Date.parse(`${value}T00:00:00Z`) - Date.parse(`${today()}T00:00:00Z`)) / 86_400_000);
  return { label: definition.label, value, chip: relativeDay(value, today()), urgent: days <= LEAD_DAYS[definition.role] };
}

function quickLookOf(target: FixtureDocument | undefined): QuickLook_Serialize | null {
  if (!target) return null;
  const folder = folderOf(target.path);
  return {
    documentId: target.id,
    path: target.path,
    fileName: fileName(target.path),
    title: target.title,
    kind: target.kind,
    readableByAi: target.kind !== "other",
    primaryFolderId: mockSession.state.folders.find((candidate) => candidate.primary)?.id ?? null,
    facts: {
      category: categories.find((category) => category.id === target.categoryId)?.name ?? null,
      tags: [...target.tags].sort(byName),
      correspondent: target.correspondent,
      documentDate: target.documentDate,
      amount: fieldFact(target, FIXTURE_FIELD_DEFINITIONS.find((definition) => definition.type === "money")),
      due: fieldFact(target, FIXTURE_FIELD_DEFINITIONS.find((definition) => definition.role === "dueDate")),
    },
    suggestion: target.suggestion ? suggestionChips(target) : null,
    categories: categoryChoices(),
    libraryTags: [...FIXTURE_TAGS].sort(byName),
    folder: folder === "" ? "/" : folder,
    details: detailsOf(target),
  };
}

function detailsOf(target: FixtureDocument): QuickLook_Serialize["details"] {
  const hints = { text: "text", number: "number", money: "amount and currency", date: "YYYY-MM-DD", boolean: "yes or no" };
  const retention = FIXTURE_RETENTION[target.id] ?? { keepUntil: null, settled: false };
  const category = categories.find((candidate) => candidate.id === target.categoryId);
  const years = category ? FIXTURE_CATEGORY_RETENTION_YEARS[category.id] : undefined;
  const yearOf = Number((target.documentDate ?? target.addedAt).slice(0, 4));
  const end = retention.keepUntil ?? (years ? `${yearOf + years}-12-31` : null);
  const links = FIXTURE_LINKS.filter((link) => link.first === target.id || link.second === target.id).flatMap((link) => {
    const other = find(link.first === target.id ? link.second : link.first);
    return other ? [{ ...candidate(other), label: link.label }] : [];
  });
  return {
    note: target.note,
    links: links.sort((first, second) => byName(first.name, second.name)),
    fields: FIXTURE_FIELD_DEFINITIONS.map((definition) => ({
      key: definition.key,
      label: `${definition.label} (${hints[definition.type]})`,
      value: FIXTURE_FIELD_VALUES[target.id]?.[definition.key] ?? "",
    })),
    ruleDisabledReason: target.correspondent ? null : "Give the document a correspondent first",
    retention: {
      status: retention.settled ? "Marked as done; never flagged again" : end ? `Retention ends on ${end}` : "No retention set",
      keepUntil: retention.keepUntil ?? "",
      category: category ? { id: category.id, label: `Keep documents of "${category.name}" for … years`, years: years ? String(years) : "" } : null,
    },
  };
}

function suggestionChips(target: FixtureDocument): string[] {
  const suggestion = FIXTURE_SUGGESTIONS[target.id];
  if (!suggestion) return [];
  const labels = Object.fromEntries(FIXTURE_FIELD_DEFINITIONS.map((definition) => [definition.key, definition.label]));
  const categoryName = suggestion.category?.path.split(" / ").pop();
  return [
    suggestion.title && `title: ${suggestion.title}`,
    suggestion.category && `${suggestion.category.isNew ? "new category" : "category"}: ${categoryName}`,
    suggestion.correspondent && `correspondent: ${suggestion.correspondent}`,
    suggestion.documentDate && `date: ${suggestion.documentDate}`,
    suggestion.targetFolder && `folder: ${suggestion.targetFolder.path}`,
    ...suggestion.tags.map((tag) => `+ ${tag}`),
    ...Object.entries(suggestion.fields).map(([key, value]) => `${labels[key] ?? key}: ${value}`),
  ].filter((chip): chip is string => Boolean(chip));
}

function saveFact(target: FixtureDocument, fact: string, typed: string): boolean {
  const value = typed.trim();
  if (fact === "title" || fact === "correspondent") {
    change(() => (target[fact] = value === "" ? null : value));
    return true;
  }
  if (fact === "date") {
    if (value !== "" && !isIsoDate(value)) return refuse("Write the date as YYYY-MM-DD, such as 2030-12-31");
    change(() => (target.documentDate = value === "" ? null : value));
    return true;
  }
  if (fact === "tag") return addTag(target, value);
  const definition = FIXTURE_FIELD_DEFINITIONS.find((candidate) => (fact === "amount" ? candidate.type === "money" : candidate.role === "dueDate"));
  if (!definition) return false;
  const parsed = value === "" ? null : parseField(definition, value);
  if (value !== "" && parsed === null) return refuse(mismatch(definition));
  change(() => {
    const values = { ...(FIXTURE_FIELD_VALUES[target.id] ?? {}) };
    if (parsed === null) delete values[definition.key];
    else values[definition.key] = parsed;
    FIXTURE_FIELD_VALUES[target.id] = values;
  });
  return true;
}

function addTag(target: FixtureDocument, value: string): boolean {
  if (value === "") return true;
  if (value.length > 60) return refuse("A tag has at most 60 characters");
  change(() => {
    const existing = FIXTURE_TAGS.find((tag) => tag.toLowerCase() === value.toLowerCase());
    if (!existing) FIXTURE_TAGS.push(value);
    const spelled = existing ?? value;
    if (!target.tags.some((tag) => tag.toLowerCase() === spelled.toLowerCase())) target.tags = [...target.tags, spelled];
  });
  return true;
}

function refuse(text: string): false {
  reportError(text);
  return false;
}

/** The value as the quick look shows it, or null when it does not fit (a light version of `FieldValues`). */
function parseField(definition: FixtureFieldDefinition, raw: string): string | null {
  if (definition.type === "date") return isIsoDate(raw) ? raw : null;
  if (definition.type !== "money") return raw;
  const match = /^([A-Za-z]{3})?\s*([\d.,' ]+?)\s*([A-Za-z]{3})?$/.exec(raw);
  if (!match || (match[1] && match[3])) return null;
  const digits = (match[2] ?? "").replace(/['\s]/g, "");
  const decimal = Math.max(digits.lastIndexOf("."), digits.lastIndexOf(","));
  const normalized = decimal < 0 ? digits : `${digits.slice(0, decimal).replace(/[.,]/g, "")}.${digits.slice(decimal + 1)}`;
  if (normalized === "" || Number.isNaN(Number(normalized))) return null;
  return `${normalized} ${(match[1] ?? match[3] ?? FIXTURE_DEFAULT_CURRENCY).toUpperCase()}`;
}

function mismatch(definition: FixtureFieldDefinition): string {
  const hints = { text: "write some text", number: "write a number", money: "write an amount, such as 12.50 EUR", date: "write a date such as 2026-12-31", boolean: "write yes or no" };
  return `"${definition.label}": ${hints[definition.type]}`;
}

function saveFields(target: FixtureDocument, fields: TypedField[]): null {
  const values: Record<string, string> = {};
  for (const field of fields) {
    const definition = FIXTURE_FIELD_DEFINITIONS.find((candidate) => candidate.key === field.key);
    if (!definition || field.value.trim() === "") continue;
    const parsed = parseField(definition, field.value.trim());
    if (parsed === null) return reportError(mismatch(definition));
    values[field.key] = parsed;
  }
  return change(() => (FIXTURE_FIELD_VALUES[target.id] = values));
}

function moveTo(target: FixtureDocument, folder: string, name: string): null {
  if (name === "" || /[<>:"/\\|?*]/.test(name)) return reportError(name === "" ? "The name is empty" : 'The name contains one of < > : " / \\ | ? *');
  const path = folder === "" ? name : `${folder}/${name}`;
  if (FIXTURE_DOCUMENTS.some((other) => other !== target && other.path.toLowerCase() === path.toLowerCase())) {
    return reportError("Another entry there already has this name (ignoring case)");
  }
  const label = `Moving ${target.path}`;
  target.path = path;
  return operation(label, `Moved to ${path}`);
}

function suggest(target: FixtureDocument, verb: "Classifying" | "Tagging"): null {
  const name = fileName(target.path);
  mockOperation(`${verb} ${name} with AI`, "ai", 4, `The AI suggested a classification for ${name}`, true);
  window.setTimeout(() => {
    const suggestion: FixtureSuggestion = {
      model: "gemma-3-12b", confidence: 0.8, title: target.title ?? name.replace(/\.[^.]+$/, ""), category: null, correspondent: target.correspondent,
      documentDate: target.documentDate, fileName: null, targetFolder: null, tags: ["2026"], fields: {}, ruleNotes: {},
    };
    FIXTURE_SUGGESTIONS[target.id] ??= suggestion;
    target.suggestion = true;
    void emit("open-review-event", { documentId: target.id });
  }, 900);
  return null;
}

function reviewOf(target: FixtureDocument): Review_Serialize | null {
  const suggestion = FIXTURE_SUGGESTIONS[target.id];
  if (!target.suggestion || !suggestion) return null;
  const rule = (part: keyof FixtureSuggestion["ruleNotes"]) => (suggestion.ruleNotes[part] ? `From rule: ${suggestion.ruleNotes[part]}` : null);
  const row = (key: string, label: string, value: string, current: string | null, extra: Partial<ReviewRow_Serialize> = {}): ReviewRow_Serialize => ({
    key, label, value, accepted: value.trim() !== "", current: current ?? "", options: [], hint: null, ...extra,
  });
  const categoryPaths = categories.map((category) => categoryPath(category.id) ?? category.name).sort(byName);
  const folders = [...new Set(FIXTURE_DOCUMENTS.flatMap((other) => other.path.split("/").slice(0, -1).map((_, index, parts) => parts.slice(0, index + 1).join("/"))))].sort(byName);
  const rows: ReviewRow_Serialize[] = [];
  if (suggestion.title !== null) rows.push(row("title", "Title", suggestion.title, target.title));
  if (suggestion.documentDate !== null) rows.push(row("documentDate", "Date (YYYY-MM-DD)", suggestion.documentDate, target.documentDate));
  if (suggestion.correspondent !== null) rows.push(row("correspondent", "Correspondent", suggestion.correspondent, target.correspondent));
  rows.push(row("category", "Category: pick one or type a new name", suggestion.category?.path ?? "", categoryPath(target.categoryId), { options: categoryPaths, hint: rule("category") }));
  if (suggestion.tags.length > 0) rows.push(row("tags", "Tags to add, comma separated", suggestion.tags.join(", "), [...target.tags].sort(byName).join(", "), { hint: rule("tags") }));
  if (suggestion.fileName !== null) rows.push({ ...row("fileName", "File name", suggestion.fileName, fileName(target.path)), accepted: false });
  const folder = suggestion.targetFolder;
  if (folder) {
    const hint = [folder.isNew ? "a new folder" : null, folder.reason ? `Why: ${folder.reason}` : rule("targetFolder")].filter(Boolean).join(" · ");
    rows.push(row("targetFolder", "Move to folder", folder.path, folderOf(target.path), { options: folders, hint: hint === "" ? null : hint }));
  }
  for (const [key, value] of Object.entries(suggestion.fields)) {
    const definition = FIXTURE_FIELD_DEFINITIONS.find((candidate) => candidate.key === key);
    rows.push(row(`field:${key}`, definition?.label ?? key, value, FIXTURE_FIELD_VALUES[target.id]?.[key] ?? null, { hint: rule("fields") }));
  }
  const source = suggestion.model === "rules" ? "your matching rules, without the AI" : suggestion.model;
  const confidence = suggestion.confidence === null ? "" : ` · ${Math.round(suggestion.confidence * 100)} % sure`;
  return { documentId: target.id, documentName: target.path, source: `From ${source}${confidence}. Only the checked values are applied.`, rows };
}

function approve(target: FixtureDocument, rows: ReviewRowDraft[]): string | null {
  const running = mockSession.state.operations.find((operation) => operation.lane === "library");
  if (running) return `Wait until "${running.label}" finishes`;
  const taken = (key: string) => {
    const value = rows.find((candidate) => candidate.key === key && candidate.accepted)?.value.trim() ?? "";
    return value === "" ? null : value;
  };
  const date = taken("documentDate");
  if (date !== null && !isIsoDate(date)) return "Write the date as YYYY-MM-DD, such as 2030-12-31";
  const category = taken("category");
  if (category !== null && category.split("/").some((name) => name.trim() === "")) return `Every level of the category "${category}" needs a name`;
  const path = target.path;
  target.title = taken("title") ?? target.title;
  target.documentDate = date ?? target.documentDate;
  target.correspondent = taken("correspondent") ?? target.correspondent;
  if (category !== null) target.categoryId = categoryId(category);
  for (const tag of (taken("tags") ?? "").split(",").map((raw) => raw.trim()).filter(Boolean)) {
    if (!target.tags.some((existing) => existing.toLowerCase() === tag.toLowerCase())) target.tags = [...target.tags, tag];
    if (!FIXTURE_TAGS.some((existing) => existing.toLowerCase() === tag.toLowerCase())) FIXTURE_TAGS.push(tag);
  }
  const folder = taken("targetFolder") ?? folderOf(target.path);
  const name = taken("fileName") ?? fileName(target.path);
  target.path = folder === "" ? name : `${folder}/${name}`;
  target.suggestion = false;
  delete FIXTURE_SUGGESTIONS[target.id];
  return operation(`Applying the suggestion for ${path}`, `Filed ${path}`);
}

/** An existing category by path or name ignoring case, else the typed path created level by level. */
function categoryId(typed: string): string {
  const wanted = typed.toLowerCase();
  const existing = categories.find((category) => categoryPath(category.id)?.toLowerCase() === wanted || category.name.toLowerCase() === wanted);
  if (existing) return existing.id;
  let parentId: string | null = null;
  for (const name of typed.split("/").map((part) => part.trim())) {
    const level = categories.find((category) => category.parentId === parentId && category.name.toLowerCase() === name.toLowerCase());
    const id: string = level?.id ?? `c-${name.toLowerCase().replace(/\W+/g, "-")}-${categories.length}`;
    if (!level) categories.push({ id, name, parentId });
    parentId = id;
  }
  return parentId ?? "";
}

function reject(target: FixtureDocument): boolean {
  if (mockSession.state.operations.some((operation) => operation.lane === "library")) return false;
  target.suggestion = false;
  delete FIXTURE_SUGGESTIONS[target.id];
  change(() => undefined);
  return true;
}
