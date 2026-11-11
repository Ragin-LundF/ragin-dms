import {emit} from "@tauri-apps/api/event";
import type {
    ApplyPreviewDto,
    PlanDto,
    PreviewDto,
    RestructureScopeDto,
    RestructureStateDto_Serialize,
    UiMessage
} from "../../ipc";
import {
    fileName,
    FIXTURE_CATEGORIES,
    FIXTURE_DOCUMENTS,
    FIXTURE_FIELD_VALUES,
    FIXTURE_RESTRUCTURE_HISTORY,
    FIXTURE_SMART_FOLDERS,
    type FixtureDocument
} from "../fixture";
import type {Handler} from "./index";
import {mockSession, updateMockSession} from "./shell";

/**
 * Runs a fake operation like `Session::perform`, with progress per step; `onDone` changes the mocked data at the end
 * and returns the message to show (or null).
 */
export function mockRun(label: string, lane: "library" | "ai", steps: number, cancellable: boolean, onDone: () => UiMessage | null) {
  const running = mockSession.state.operations.find((operation) => operation.lane === lane);
  if (running) throw { code: "busy", message: `Wait until "${running.label}" finishes` };
  updateMockSession((state) => ({ ...state, operations: [...state.operations, { label, lane, cancellable }] }));
  let done = 0;
  const timer = window.setInterval(() => {
    done++;
    void emit("progress-event", { lanes: [{ lane, progress: { step: label, done, total: steps, detail: null } }] });
    if (done < steps) return;
    window.clearInterval(timer);
    void emit("progress-event", { lanes: [] });
    const message = onDone();
    if (message) void emit("message-event", { message });
    updateMockSession((state) => ({ ...state, operations: state.operations.filter((operation) => operation.lane !== lane), revision: state.revision + 1 }));
  }, 250);
}

type Move = { from: string; to: string };
type SkipReason = "without category" | "without date" | "without correspondent" | "without the field value" | "in the inbox";

const DEFAULT_TEMPLATE = "{category}/{year|Undated}";
const PLACEHOLDERS = ["category", "category.top", "year", "month", "correspondent", "kind", "field:due"];

const restructure: { saved: string | null; moves: Move[]; plan: PlanDto | null; emptyFolders: string[]; history: string[] } = {
  saved: null,
  moves: [],
  plan: null,
  emptyFolders: [],
  history: [...FIXTURE_RESTRUCTURE_HISTORY],
};

function folders(): string[] {
  const all = new Set<string>();
  for (const document of FIXTURE_DOCUMENTS) {
    const parts = document.path.split("/").slice(0, -1);
    parts.forEach((_, index) => all.add(parts.slice(0, index + 1).join("/")));
  }
  return [...all].sort((first, second) => first.toLowerCase().localeCompare(second.toLowerCase()));
}

function state(): RestructureStateDto_Serialize {
  return {
    scopes: [
      { label: "Whole library", scope: { kind: "folder", path: "" } },
      ...FIXTURE_SMART_FOLDERS.map((search) => ({ label: `Smart folder: ${search.name}`, scope: { kind: "smartFolder" as const, id: search.id } })),
      ...folders().map((folder) => ({ label: folder, scope: { kind: "folder" as const, path: folder } })),
    ],
    savedTemplate: restructure.saved,
    initialTemplate: restructure.saved ?? DEFAULT_TEMPLATE,
    placeholders: PLACEHOLDERS,
    plan: restructure.plan,
    emptyFolders: restructure.emptyFolders,
  };
}

/** The template's problem in the words of the real presenter, or null. */
function problemOf(template: string): string | null {
  if (template.trim() === "") return "Write a template, such as {category}/{year}";
  const tokens = [...template.matchAll(/\{([^{}]*)\}/g)];
  if ((template.match(/\{/g) ?? []).length !== (template.match(/\}/g) ?? []).length) return "A { has no closing }";
  for (const [, inner = ""] of tokens) {
    const name = (inner.split("|")[0] ?? "").trim();
    if (name.startsWith("field:")) {
      if (!/^field:[a-z][a-z0-9_-]*$/.test(name)) return "Write fields as {field:key}, such as {field:due}";
    } else if (!PLACEHOLDERS.slice(0, 6).includes(name)) return "Unknown placeholder; use the chips below";
  }
  if (/[<>:"\\|?*]/.test(template.replace(/\{[^{}]*\}/g, ""))) return 'A folder name is not allowed or contains one of < > : " \\ | ? *';
  return null;
}

function categoryPath(id: string | null): string[] {
  const category = FIXTURE_CATEGORIES.find((candidate) => candidate.id === id);
  return category ? [...categoryPath(category.parentId), category.name] : [];
}

function value(document: FixtureDocument, name: string): string | null {
  const date = document.documentDate;
  if (name === "category") return categoryPath(document.categoryId).join("/") || null;
  if (name === "category.top") return categoryPath(document.categoryId)[0] ?? null;
  if (name === "year") return date?.slice(0, 4) ?? null;
  if (name === "month") return date?.slice(5, 7) ?? null;
  if (name === "correspondent") return document.correspondent;
  if (name === "kind") return document.kind.charAt(0).toUpperCase() + document.kind.slice(1);
  return FIXTURE_FIELD_VALUES[document.id]?.[name.slice("field:".length)] ?? null;
}

const SKIP: Record<string, SkipReason> = { category: "without category", "category.top": "without category", year: "without date", month: "without date", correspondent: "without correspondent" };

function render(document: FixtureDocument, template: string): string | SkipReason {
  let skipped: SkipReason | null = null;
  const folder = template.replace(/\{([^{}]*)\}/g, (_, inner: string) => {
    const [name = "", fallback] = inner.split("|").map((part) => part.trim());
    const found = value(document, name) ?? fallback ?? null;
    if (found === null) skipped ??= SKIP[name] ?? "without the field value";
    return found ?? "";
  });
  return skipped ?? folder.split("/").map((part) => part.trim()).filter(Boolean).join("/");
}

function inScope(document: FixtureDocument, scope: RestructureScopeDto): boolean {
  if (scope.kind === "folder") return scope.path === "" || document.path.startsWith(`${scope.path}/`);
  if (scope.id === "s-car") return document.tags.includes("car");
  return document.tags.includes("contract");
}

function preview(scope: RestructureScopeDto, template: string, naming: "keep" | "dateAndTitle"): PreviewDto {
  const problem = problemOf(template);
  if (problem) return { kind: "problem", text: problem };
  const moves: Move[] = [];
  const skipped = new Map<SkipReason, number>();
  let unchanged = 0;
  for (const document of FIXTURE_DOCUMENTS.filter((candidate) => inScope(candidate, scope))) {
    const target = document.path.startsWith("Inbox/") ? "in the inbox" : render(document, template);
    if (target === "in the inbox" || SKIPPED.has(target)) {
      skipped.set(target as SkipReason, (skipped.get(target as SkipReason) ?? 0) + 1);
      continue;
    }
    const extension = fileName(document.path).split(".").pop();
    const name = naming === "dateAndTitle" && document.title && document.documentDate ? `${document.documentDate} ${document.title}.${extension}` : fileName(document.path);
    const to = target ? `${target}/${name}` : name;
    if (to === document.path) unchanged++;
    else moves.push({ from: document.path, to });
  }
  const groups = new Map<string, Move[]>();
  for (const move of moves) {
    const folder = move.to.split("/").slice(0, -1).join("/");
    groups.set(folder, [...(groups.get(folder) ?? []), move]);
  }
  restructure.moves = moves;
  restructure.plan = {
    groups: [...groups.entries()]
      .sort(([first], [second]) => (first < second ? -1 : 1))
      .map(([folder, grouped]) => ({ folder, count: grouped.length, samples: grouped.slice(0, 3).map((move) => `${move.from} → ${fileName(move.to)}`) })),
    unchanged,
    skipped: [...skipped.entries()].map(([reason, count]) => `${count} ${reason}`),
    renamed: [],
  };
  return { kind: "planned", plan: restructure.plan };
}

const SKIPPED = new Set<string>(["without category", "without date", "without correspondent", "without the field value"]);

function chosen(included: string[]): Move[] {
  return restructure.moves.filter((move) => included.includes(move.to.split("/").slice(0, -1).join("/")));
}

function recordName(): string {
  return new Date().toISOString().replace(/[-:]/g, "").replace("T", "-").slice(0, 15);
}

/** Mocked commands of features::restructure. */
export const restructureHandlers: Record<string, Handler> = {
  restructure_state: () => state(),
  restructure_history: () => restructure.history,
  preview_restructure: (args) => preview(args.scope as RestructureScopeDto, args.template as string, args.naming as "keep" | "dateAndTitle"),
  restructure_apply_preview: (args): ApplyPreviewDto => {
    const moves = chosen(args.includedFolders as string[]);
    const affected = moves.slice(0, 12).map((move) => `${move.from} → ${move.to}`);
    if (moves.length > 12) affected.push(`… and ${moves.length - 12} more`);
    return { moves: moves.length, affected };
  },
  apply_restructure: (args) => {
    const moves = chosen(args.includedFolders as string[]);
    mockRun(`Restructuring ${moves.length} documents`, "library", Math.min(moves.length, 8), true, () => {
      restructure.plan = null;
      restructure.history = [recordName(), ...restructure.history];
      restructure.emptyFolders = ["Insurance", "Health"];
      return { text: `Moved ${moves.length} documents`, isError: false };
    });
    return null;
  },
  reverse_restructure: (args) => {
    mockRun(`Reversing restructure ${args.name as string}`, "library", 5, true, () => {
      restructure.plan = null;
      restructure.history = [recordName(), ...restructure.history];
      return { text: "Moved 5 documents; 1 changed since and stayed", isError: false };
    });
    return null;
  },
  remove_empty_folders: (args) => {
    restructure.emptyFolders = [];
    mockRun(`Removing ${(args.folders as string[]).length} empty folders`, "library", 2, false, () => ({ text: "Removed empty folders", isError: false }));
    return null;
  },
  dismiss_empty_folders: () => {
    restructure.emptyFolders = [];
    return null;
  },
  save_path_template: (args) => {
    const template = (args.template as string).trim();
    mockRun("Saving the path template", "library", 1, false, () => {
      restructure.saved = template;
      return { text: `Saved ${template} as the default template`, isError: false };
    });
    return null;
  },
};
