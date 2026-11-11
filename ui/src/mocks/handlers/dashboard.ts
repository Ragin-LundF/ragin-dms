import type {
    AttentionRowDto_Serialize,
    AttentionSeverity,
    ChartPointDto,
    DashboardAction,
    DashboardButtonDto_Serialize as Button,
    DashboardViewDto_Serialize,
    DuplicateCopyDto_Serialize,
    TaskGroupDto_Serialize,
    TaskGroupKind,
    TaskRowDto_Serialize,
} from "../../ipc";
import {fileName, FIXTURE_CATEGORIES, FIXTURE_DOCUMENTS, FIXTURE_FOLDERS, type FixtureDocument} from "../fixture";
import type {Handler} from "./index";
import {mockOperation, mockSession} from "./shell";

/**
 * Mocked commands of features::dashboard: a dashboard built from the fixture with one task of every kind, so
 * every row, button and dialog can be tried. Answering a task removes it, like Rust forgetting a resolved change.
 */

const { primary: PRIMARY, sync: SYNC } = FIXTURE_FOLDERS;
const FOLDER_NAMES: Record<string, string> = { [PRIMARY]: "Documents", [SYNC]: "Cloud" };
const DAY = 86_400_000;

type MockTask = { group: TaskGroupKind; row: TaskRowDto_Serialize };

function button(label: string, action: DashboardAction, more: Partial<Button> = {}): Button {
  return { label, icon: null, style: "outlined", disabledReason: null, lane: null, action, confirm: null, ...more };
}

const waits = (buttons: Button[]) => buttons.map((entry) => ({ ...entry, lane: "library" as const }));
const dismiss = (taskKey: string) => button("Dismiss", { kind: "dismiss", taskKey }, { style: "text" });
const open = (label: string, folderId: string, path: string) =>
  button(label, { kind: "open", folderId, path }, { icon: "open_in_new", style: "text" });
const everywhere = (path: string) => [`Documents: ${path}`, `Cloud: ${path}`];
const ask = (title: string, message: string, affected: string[], confirmLabel: string) => ({ title, message, affected, confirmLabel });

function row(key: string, title: string, detail: string, buttons: Button[], opens: Button[], copies: DuplicateCopyDto_Serialize[] = []): TaskRowDto_Serialize {
  const first = opens[0]?.action ?? copies[0]?.open.action;
  const preview = first?.kind === "open" ? { folderId: first.folderId, path: first.path } : null;
  return { key, title, detail, buttons: waits(buttons), opens: copies.length > 0 ? [] : opens, preview, copies };
}

const TRASH_NEW = "The file is not in the library. It goes to the trash in the folders that have it.";
const KEEP_VERSION = (name: string) => `Every other version of this file goes to the trash and is replaced by the one in ${name}.`;

function initialTasks(): MockTask[] {
  const suggested = FIXTURE_DOCUMENTS.filter((document) => document.suggestion && !document.path.startsWith("Inbox/"));
  return [
    ...suggested.map((document) => ({
      group: "suggestions" as const,
      row: row(
        `suggestion:${document.id}`,
        `Suggestion for ${document.path}`,
        `Title: ${document.title ?? fileName(document.path).replace(/\.\w+$/, "")} · Category: Health · Tags: 2026`,
        [
          button("Review", { kind: "review", documentId: document.id }, { icon: "auto_awesome", style: "tonal" }),
          button("Reject", { kind: "reject", documentId: document.id }, { style: "text" }),
        ],
        [open("Open", PRIMARY, document.path)],
      ),
    })),
    {
      group: "deadlines",
      row: row("deadline:d-010:cancel_by:2026-03-31", "Cancel by on 2026-03-31: Insurance/Car insurance policy.pdf", "Dismiss once it is handled; a new date brings the task back", [dismiss("deadline:d-010:cancel_by:2026-03-31")], [open("Open", PRIMARY, "Insurance/Car insurance policy.pdf")]),
    },
    {
      group: "retention",
      row: row(
        "retention:d-012:2026-01-01",
        "Retention ended: Health/Vaccination record.pdf",
        "You chose to keep it until 2026-01-01. Move it to the trash, or keep it and mark this as done.",
        [
          button("Mark as done", { kind: "settleRetention", documentIds: ["d-012"] }, { icon: "check", style: "tonal" }),
          button("Move to trash…", { kind: "trashRetained", documentIds: ["d-012"] }, {
            icon: "delete",
            confirm: ask("Move 1 documents to the trash", "You chose to keep them until a date that has passed. They go to the trash in every library folder and leave the catalog.", everywhere("Health/Vaccination record.pdf"), "Move to trash"),
          }),
        ],
        [open("Open", PRIMARY, "Health/Vaccination record.pdf")],
      ),
    },
    {
      group: "learnedRules",
      row: row("rule:northbank|c-bank|2026", "Rule for Northbank", "From 3 approved documents · Category: Bank statements · Tags: 2026", [
        button("Accept rule", { kind: "acceptRule", taskKey: "rule:northbank|c-bank|2026" }, { icon: "check", style: "tonal" }),
        button("Reject", { kind: "dismiss", taskKey: "rule:northbank|c-bank|2026" }, { style: "text" }),
      ], []),
    },
    {
      group: "newFiles",
      row: row("new:Home/Garden plan.pdf", "New file Home/Garden plan.pdf", "In Documents, Cloud, 182.4 KB", [
        button("Adopt", { kind: "adopt", taskKey: "new:Home/Garden plan.pdf" }, { icon: "check", style: "tonal" }),
        button("Move to trash…", { kind: "trashNewFile", taskKey: "new:Home/Garden plan.pdf" }, { confirm: ask("Move to trash", TRASH_NEW, everywhere("Home/Garden plan.pdf"), "Move to trash") }),
        dismiss("new:Home/Garden plan.pdf"),
      ], [open("Open", PRIMARY, "Home/Garden plan.pdf")]),
    },
    {
      group: "newFiles",
      row: row("new:Downloads/unread.pdf", "New file Downloads/unread.pdf", "In Cloud, 12.0 KB", [
        button("Adopt", { kind: "adopt", taskKey: "new:Downloads/unread.pdf" }, { icon: "check", style: "tonal", disabledReason: "The file was not read yet; scan again" }),
        button("Move to trash…", { kind: "trashNewFile", taskKey: "new:Downloads/unread.pdf" }, { confirm: ask("Move to trash", TRASH_NEW, ["Cloud: Downloads/unread.pdf"], "Move to trash") }),
        dismiss("new:Downloads/unread.pdf"),
      ], [open("Open", SYNC, "Downloads/unread.pdf")]),
    },
    {
      group: "missing",
      row: row("missing:d-009", "Missing Home/Floor plan.png", "Missing in Cloud; still in Documents", [
        button("Restore", { kind: "restore", taskKey: "missing:d-009" }, { icon: "restore", style: "tonal" }),
        button("Move to trash…", { kind: "deleteEverywhere", taskKey: "missing:d-009" }, {
          confirm: ask("Delete everywhere", "The file goes to the trash in every library folder and leaves the catalog.", everywhere("Home/Floor plan.png"), "Move to trash"),
        }),
        dismiss("missing:d-009"),
      ], [open("Open", PRIMARY, "Home/Floor plan.png")]),
    },
    {
      group: "modified",
      row: row("modified:d-014:f-sync", "Changed outside the tool: Budget 2026.xlsx", "The content in Cloud differs from the catalog", [
        button("Keep new content…", { kind: "keepVersion", taskKey: "modified:d-014:f-sync", folderId: SYNC }, { confirm: ask("Keep the version of Cloud", KEEP_VERSION("Cloud"), ["Documents: Budget 2026.xlsx"], "Keep this version") }),
        button("Restore previous…", { kind: "keepVersion", taskKey: "modified:d-014:f-sync", folderId: PRIMARY }, { confirm: ask("Keep the version of Documents", KEEP_VERSION("Documents"), ["Cloud: Budget 2026.xlsx"], "Keep this version") }),
        dismiss("modified:d-014:f-sync"),
      ], [open("Open new version", SYNC, "Budget 2026.xlsx"), open("Open previous version", PRIMARY, "Budget 2026.xlsx")]),
    },
    {
      group: "moved",
      row: row("moved:d-007:f-sync", "Other name in Cloud: Home/Power invoice March.pdf", "The catalog and the other folders call it Home/Electricity invoice March.pdf", [
        button("Rename to Electricity invoice March.pdf here", { kind: "resolveMove", taskKey: "moved:d-007:f-sync", choice: "undo" }, { icon: "sync", style: "tonal" }),
        button("Use Power invoice March.pdf everywhere", { kind: "resolveMove", taskKey: "moved:d-007:f-sync", choice: "apply" }),
        dismiss("moved:d-007:f-sync"),
      ], [open("Open", SYNC, "Home/Power invoice March.pdf")]),
    },
    {
      group: "duplicates",
      row: row("duplicates:aa11", "2 copies of the same file", "Same size and SHA-256 checksum. Keep one; the others go to the trash in every folder.", [dismiss("duplicates:aa11")], [], [
        copy("Finance/Bank statement 2026-01.pdf", "d-003", "Recommended · Category: Bank statements · Tags: 2026", "Finance/Bank statement 2026-01 (1).pdf"),
        copy("Finance/Bank statement 2026-01 (1).pdf", "d-021", "Nothing assigned", "Finance/Bank statement 2026-01.pdf"),
      ]),
    },
    {
      group: "duplicates",
      row: row("new:Inbox/receipt copy.jpg", "Copy of a document: Inbox/receipt copy.jpg", "In Documents, 694.6 KB; same size and checksum as Inbox/receipt.jpg", [
        button("Move copy to trash…", { kind: "trashDuplicateFile", taskKey: "new:Inbox/receipt copy.jpg" }, {
          icon: "delete",
          style: "tonal",
          confirm: ask("Move copy to trash", "The file has exactly the same size and SHA-256 checksum as Inbox/receipt.jpg. Right before it goes to the trash it is compared with it byte by byte once more, in each folder.", ["Documents: Inbox/receipt copy.jpg"], "Move to trash"),
        }),
        button("Adopt anyway", { kind: "adopt", taskKey: "new:Inbox/receipt copy.jpg" }),
        dismiss("new:Inbox/receipt copy.jpg"),
      ], [open("Open", PRIMARY, "Inbox/receipt copy.jpg")]),
    },
    {
      group: "conflicts",
      row: row("conflict:Notes 2.txt", "Different content: Notes 2.txt", "2 versions in Documents, Cloud", [
        ...[PRIMARY, SYNC].map((folderId) => {
          const other = FOLDER_NAMES[folderId === PRIMARY ? SYNC : PRIMARY] ?? "";
          const name = FOLDER_NAMES[folderId] ?? folderId;
          return button(`Keep ${name}…`, { kind: "keepVersion", taskKey: "conflict:Notes 2.txt", folderId }, { confirm: ask(`Keep the version of ${name}`, KEEP_VERSION(name), [`${other}: Notes 2.txt`], "Keep this version") });
        }),
        button("Keep both", { kind: "keepBoth", taskKey: "conflict:Notes 2.txt", folderId: SYNC }),
        dismiss("conflict:Notes 2.txt"),
      ], [open("Open Documents version", PRIMARY, "Notes 2.txt"), open("Open Cloud version", SYNC, "Notes 2.txt")]),
    },
    {
      group: "catalogMerge",
      row: row("merge:category:c-home", "Catalog copies differed for category Home", "Kept \"Home\", dropped \"House\"", [button("OK", { kind: "dismiss", taskKey: "merge:category:c-home" })], []),
    },
    {
      group: "failedOperations",
      row: row("retry:f-sync", "2 operation(s) waiting for Cloud", "They are applied in order once the folder is reachable", [
        button("Retry", { kind: "retry" }, { icon: "refresh", style: "tonal" }),
        dismiss("retry:f-sync"),
      ], []),
    },
    {
      group: "restructure",
      row: row("restructure:Finance:42", "42 files without structure in Finance", "The restructure assistant can sort them into subfolders", [
        button("Restructure…", { kind: "restructure", folder: "Finance" }, { icon: "account_tree", style: "tonal" }),
        dismiss("restructure:Finance:42"),
      ], []),
    },
  ];
}

function copy(path: string, documentId: string, detail: string, other: string): DuplicateCopyDto_Serialize {
  const recommended = detail.startsWith("Recommended");
  return {
    path,
    detail,
    open: open("Open", PRIMARY, path),
    keep: button("Keep…", { kind: "keepDuplicate", taskKey: "duplicates:aa11", documentId }, {
      style: recommended ? "tonal" : "outlined",
      lane: "library",
      confirm: ask(
        `Keep ${path}`,
        `The other copies have exactly the same size and SHA-256 checksum. Right before they go to the trash in every folder, each is compared byte by byte once more. Their categories, tags and title move to ${fileName(path)} where it has none.`,
        everywhere(other),
        "Move copies to trash",
      ),
    }),
  };
}

let tasks = initialTasks();

const GROUPS: { kind: TaskGroupKind; label: string; icon: string; severity: AttentionSeverity }[] = [
  { kind: "suggestions", label: "Suggestions", icon: "auto_awesome", severity: "info" },
  { kind: "deadlines", label: "Deadlines", icon: "event", severity: "passed" },
  { kind: "retention", label: "Retention ended", icon: "inventory_2", severity: "passed" },
  { kind: "learnedRules", label: "Learned rules", icon: "rule", severity: "info" },
  { kind: "newFiles", label: "New files", icon: "new_releases", severity: "info" },
  { kind: "missing", label: "Missing files", icon: "find_in_page", severity: "soon" },
  { kind: "modified", label: "Changed outside", icon: "edit", severity: "soon" },
  { kind: "moved", label: "Different names", icon: "swap_horiz", severity: "soon" },
  { kind: "duplicates", label: "Duplicates", icon: "content_copy", severity: "soon" },
  { kind: "conflicts", label: "Conflicts", icon: "error_outline", severity: "soon" },
  { kind: "catalogMerge", label: "Catalog merge", icon: "merge_type", severity: "soon" },
  { kind: "failedOperations", label: "Failed operations", icon: "sync_problem", severity: "soon" },
  { kind: "restructure", label: "Folder structure", icon: "account_tree", severity: "info" },
];
const SEVERITY_LABEL: Record<AttentionSeverity, string> = { passed: "Date passed", soon: "Decide soon", info: "To do" };
const SEVERITY_ORDER: AttentionSeverity[] = ["passed", "soon", "info"];

function bulkActions(kind: TaskGroupKind, rows: TaskRowDto_Serialize[]): Button[] {
  const count = rows.length;
  const buttons: Partial<Record<TaskGroupKind, Button[]>> = {
    newFiles: [button(`Adopt all (${rows.filter((entry) => !entry.buttons[0]?.disabledReason).length})`, { kind: "adoptAll" })],
    missing: [button(`Restore all (${count})`, { kind: "restoreAll" })],
    moved: [button(`Rename all to match (${count})`, { kind: "renameAll" })],
    duplicates: [
      button(`Remove all duplicates (${count})…`, { kind: "removeAllDuplicates" }, {
        confirm: ask(
          "Remove all duplicates",
          "Each file has exactly the same size and SHA-256 checksum as the document named after it, and is compared with it byte by byte once more right before it goes to the trash, in every folder. Where a document keeps one of its copies, it takes over the copy's categories and tags.",
          ["Inbox/receipt copy.jpg (copy of Inbox/receipt.jpg)", "Finance/Bank statement 2026-01 (1).pdf (copy of Finance/Bank statement 2026-01.pdf)"],
          "Move duplicates to trash",
        ),
      }),
    ],
    retention: [
      button(`Mark all as done (${count})`, { kind: "settleRetention", documentIds: ["d-012"] }),
      button(`Move all to trash (${count})…`, { kind: "trashRetained", documentIds: ["d-012"] }),
    ],
  };
  return waits(buttons[kind] ?? []);
}

function groups(): TaskGroupDto_Serialize[] {
  return GROUPS.map((group) => ({ group, rows: tasks.filter((task) => task.group === group.kind).map((task) => task.row) }))
    .filter(({ rows }) => rows.length > 0)
    .map(({ group, rows }) => ({ group: group.kind, label: group.label, icon: group.icon, rows, bulkActions: bulkActions(group.kind, rows) }));
}

function attentionRow(key: string, severity: AttentionSeverity, title: string, detail: string, icon: string, more: Partial<AttentionRowDto_Serialize> = {}): AttentionRowDto_Serialize {
  return { key, severity, title, detail: `${SEVERITY_LABEL[severity]} · ${detail}`, icon, preview: null, buttons: [], opens: [], copies: [], ...more };
}

function attention(taskGroups: TaskGroupDto_Serialize[], scanCount: number | null) {
  const inbox = FIXTURE_DOCUMENTS.filter((document) => document.path.startsWith("Inbox/")).length;
  const all: AttentionRowDto_Serialize[] = [
    attentionRow("inbox", "info", `${inbox} new documents in the inbox`, "In Inbox of your library folders", "move_to_inbox", {
      buttons: [button("Analyze all", { kind: "analyzeInbox" }, { icon: "auto_awesome", style: "tonal", lane: "ai" })],
      opens: [button("Open inbox", { kind: "openInbox" }, { icon: "arrow_forward" })],
    }),
    attentionRow("upcoming:d-006:Cancel by:2026-12-31", "soon", "Rental contract", "Cancel by on 2026-12-31", "event", {
      preview: { folderId: PRIMARY, path: "Home/Rental contract.pdf" },
      opens: [button("Open", { kind: "openDocument", documentId: "d-006" })],
    }),
    ...taskGroups.flatMap((group) => {
      const severity = GROUPS.find((entry) => entry.kind === group.group)?.severity ?? "info";
      return group.rows.map((task) => attentionRow(`task:${task.key}`, severity, task.title, task.detail, group.icon, { preview: task.preview, buttons: task.buttons, opens: task.opens, copies: task.copies }));
    }),
  ];
  if (scanCount) {
    all.push(attentionRow("scans", "info", `${scanCount} scans without text`, "PDFs search cannot read, until they get a text layer", "document_scanner", {
      buttons: waits([button(`Make ${scanCount} searchable`, { kind: "makeScansSearchable" }, { style: "tonal" }), button("Look again", { kind: "findScans" }, { style: "text" })]),
    }));
  }
  const headerButtons = [
    ...(scanCount ? [] : waits([button("Find scans without text", { kind: "findScans" }, { style: "text" })])),
    button("Export .ics", { kind: "exportCalendar" }, { style: "text" }),
  ];
  const rows = [...all].sort((first, second) => SEVERITY_ORDER.indexOf(first.severity) - SEVERITY_ORDER.indexOf(second.severity)).slice(0, 5);
  return { rows, total: all.length, headerButtons };
}

function ranked(entries: [string, string][], limit: number): ChartPointDto[] {
  const counts = new Map<string, ChartPointDto>();
  for (const [id, label] of entries) counts.set(id, { id, label, value: (counts.get(id)?.value ?? 0) + 1 });
  return [...counts.values()]
    .sort((first, second) => second.value - first.value || first.label.toLowerCase().localeCompare(second.label.toLowerCase()))
    .slice(0, limit);
}

function yearOf(document: FixtureDocument): number {
  return Number((document.documentDate ?? document.addedAt).slice(0, 4));
}

function byYear(documents: FixtureDocument[]): ChartPointDto[] {
  const thisYear = new Date().getFullYear();
  const oldest = Math.min(...documents.map(yearOf));
  const first = Math.max(oldest, thisYear - 7);
  return Array.from({ length: thisYear - first + 1 }, (_, index) => first + index).map((year) => {
    const folded = year === first && oldest < first;
    const value = documents.filter((document) => (folded ? yearOf(document) <= year : yearOf(document) === year)).length;
    const label = folded ? `≤${year}` : String(year);
    return { id: label, label, value };
  });
}

const pad = (value: number) => String(value).padStart(2, "0");
const stamp = (date: Date) => `${pad(date.getDate())}.${pad(date.getMonth() + 1)}. ${pad(date.getHours())}:${pad(date.getMinutes())}`;

function recent(documents: FixtureDocument[], now: number) {
  const categoryName = (id: string | null) => FIXTURE_CATEGORIES.find((category) => category.id === id)?.name ?? null;
  return [...documents]
    .sort((first, second) => Date.parse(second.addedAt) - Date.parse(first.addedAt))
    .slice(0, 6)
    .map((document) => {
      const days = Math.round((now - Date.parse(document.addedAt)) / DAY);
      const folder = document.path.includes("/") ? document.path.slice(0, document.path.lastIndexOf("/")) : "Top level";
      const category = categoryName(document.categoryId);
      return {
        id: document.id,
        path: document.path,
        title: document.title ?? fileName(document.path),
        subtitle: category ? `${folder} · ${category}` : folder,
        kind: document.kind,
        added: days === 0 ? "today" : days === 1 ? "yesterday" : `${days} days ago`,
        addedOn: document.addedAt.slice(0, 10),
      };
    });
}

let scansLookedFor = false;

function view(scanCount: number | null): DashboardViewDto_Serialize {
  const documents = FIXTURE_DOCUMENTS;
  const now = Date.now();
  const sizeText = `${(documents.reduce((sum, document) => sum + document.size, 0) / 1024 / 1024).toFixed(1)} MB`;
  const uncategorized = documents.filter((document) => document.categoryId === null).length;
  const share = (documents.length - uncategorized) / documents.length;
  const recently = documents.filter((document) => Date.parse(document.addedAt) > now - 30 * DAY).length;
  const lastScan = mockSession.state.lastScanAt ? new Date(mockSession.state.lastScanAt) : null;
  const taskGroups = groups();
  return {
    headerLine: `${documents.length} documents · ${sizeText} · 2 library folders · ${lastScan ? `last scan ${stamp(lastScan)}` : "not scanned yet"}`,
    stats: [
      {
        label: "Documents",
        value: String(documents.length),
        count: documents.length,
        caption: recently > 0 ? `+${recently} in the last 30 days` : "None added lately",
        meter: null,
        trend: Array.from({ length: 30 }, (_, index) => documents.filter((document) => Date.parse(document.addedAt) <= now - (29 - index) * DAY).length),
        link: null,
      },
      { label: "Categorized", value: `${Math.round(share * 100)} %`, count: null, caption: `${uncategorized} without a category`, meter: share, trend: [], link: uncategorized > 0 ? "uncategorized" : null },
      { label: "Inbox", value: "3", count: null, caption: "Waiting to be filed", meter: null, trend: [], link: "inbox" },
      { label: "Storage", value: sizeText, count: null, caption: "Kept in each of 2 library folders", meter: null, trend: [], link: null },
    ],
    aiBand: mockSession.state.aiEnabled ? { headline: "9 documents have no tags yet, 6 have no category", classifyLabel: "Classify 6", tagLabel: "Tag 9 with AI" } : null,
    attention: attention(taskGroups, scansLookedFor ? scanCount : null),
    recent: recent(documents, now),
    primaryFolderId: PRIMARY,
    categories: [
      ...ranked(documents.flatMap((document) => {
        const category = FIXTURE_CATEGORIES.find((entry) => entry.id === document.categoryId);
        return category ? [[category.id, category.name] as [string, string]] : [];
      }), 8),
      ...(uncategorized > 0 ? [{ id: "", label: "Uncategorized", value: uncategorized }] : []),
    ],
    tags: ranked(documents.flatMap((document) => document.tags.map((tag): [string, string] => [tag, tag])), 24),
    byYear: byYear(documents),
    byKind: (["pdf", "image", "office", "other"] as const).map((kind) => ({ kind, count: documents.filter((document) => document.kind === kind).length })).filter((entry) => entry.count > 0),
    correspondents: ranked(documents.flatMap((document) => (document.correspondent ? [[document.correspondent, document.correspondent] as [string, string]] : [])), 6),
    folders: [
      { title: "Documents · primary", location: "/Synthetic/Documents", primary: true, health: "reachable", status: "reachable" },
      { title: "Cloud", location: "/Synthetic/Cloud/Documents", primary: false, health: "offline", status: "offline, 2 pending" },
    ],
    activity: (
      [
        ["Added", "Inbox/scan-2026-03-08.pdf", 0.1],
        ["Approved suggestion", "Finance/Bank statement 2026-03.pdf", 0.5],
        ["Moved", "Home/Rental contract.pdf", 1.2],
        ["Synchronized", "Cloud", 2],
        ["Edited PDF", "Health/Lab results.pdf", 3.4],
      ] as const
    ).map(([verb, subject, daysAgo]) => ({ line: `${verb} ${subject}`, time: stamp(new Date(now - daysAgo * DAY)) })),
    groups: taskGroups,
  };
}

const LABELS: Partial<Record<DashboardAction["kind"], string>> = {
  reject: "Rejecting the suggestion",
  dismiss: "Dismissing the task",
  adopt: "Adopting the file",
  trashNewFile: "Trashing the file",
  trashDuplicateFile: "Trashing the copy",
  restore: "Restoring the file",
  deleteEverywhere: "Deleting the file",
  keepVersion: "Keeping the version",
  keepBoth: "Keeping both versions",
  resolveMove: "Resolving the move",
  keepDuplicate: "Removing copies",
  settleRetention: "Marking the retention as done",
  trashRetained: "Moving documents to the trash",
  acceptRule: "Adding the rule",
  adoptAll: "Adopting new files",
  restoreAll: "Restoring files",
  renameAll: "Renaming files to match",
  removeAllDuplicates: "Removing duplicates",
};

const BULK_GROUP: Partial<Record<DashboardAction["kind"], TaskGroupKind>> = {
  adoptAll: "newFiles",
  restoreAll: "missing",
  renameAll: "moved",
  removeAllDuplicates: "duplicates",
};

function runAction(action: DashboardAction) {
  if (action.kind === "analyzeInbox") return mockOperation("Analyzing the inbox", "ai", 6, "Analyzed the inbox", true);
  const label = LABELS[action.kind];
  if (!label) throw { code: "invalid_input", message: "The window handles this action itself" };
  mockOperation(label, "library", 3, `${label}: done`);
  const group = BULK_GROUP[action.kind];
  tasks = tasks.filter((task) => {
    if (group) return task.group !== group;
    if ("taskKey" in action) return task.row.key !== action.taskKey;
    if ("documentId" in action) return task.row.key !== `suggestion:${action.documentId}`;
    if ("documentIds" in action) return !action.documentIds.some((id) => task.row.key.includes(id));
    return true;
  });
  return null;
}

export const dashboardHandlers: Record<string, Handler> = {
  dashboard_view: ({ scanCount }) => view((scanCount as number | null) ?? null),
  run_dashboard_action: ({ action }) => runAction(action as DashboardAction),
  ai_tag_all: () => mockOperation("Tagging with AI", "ai", 9, "The AI suggested tags for 7 documents; review them under Open tasks", true),
  ai_classify_all: () => mockOperation("Classifying with AI", "ai", 6, "The AI classified 5 documents; review them under Open tasks", true),
  find_scans_without_text: () => {
    scansLookedFor = true;
    return ["d-016", "d-017", "d-019"];
  },
  export_calendar: () => mockOperation("Saving the calendar", "library", 1, "Saved /Synthetic/deadlines.ics"),
  open_task_file: () => null,
};
