import {emit} from "@tauri-apps/api/event";
import {mockConvertFileSrc} from "@tauri-apps/api/mocks";
import type {
    LibraryFacetDto,
    LibraryFacetOptionDto_Serialize as Option,
    LibraryFilterDto_Serialize as Filter,
    LibraryFolderEntryDto,
    LibraryFolderViewDto_Serialize,
    LibraryLayoutDto,
    LibraryParamsDto_Serialize as Params,
    LibraryRowDto_Serialize as Row,
    LibraryStateDto_Serialize,
    LibraryTokenDto_Serialize as Token,
    LibraryViewDto,
} from "../../ipc";
import {
    fileName,
    FIXTURE_CATEGORIES,
    FIXTURE_DIRECTORIES,
    FIXTURE_DOCUMENTS,
    FIXTURE_TAGS,
    type FixtureDocument
} from "../fixture";
import type {Handler} from "./index";
import {mockOperation, mockSession} from "./shell";

/**
 * Mocked commands of features::library: a small presenter over the fixture (the real rules live in Rust and are
 * tested there), and actions that change the fixture through fake operations, so the screen can be tried in full.
 */

// Thumbnails ask for dms:// URLs, which a browser cannot load: every picture shows its fallback.
// shortcut: belongs into mocks/index.ts (shell).
mockConvertFileSrc("macos");

type SavedView = { id: string; name: string; params: Params };

const ALL: LibraryViewDto = { kind: "folder", path: "" };
const saved: SavedView[] = [
  { id: "s-2025", name: "Tagged 2025", params: { view: ALL, filters: [{ kind: "tag", tag: "2025" }], query: "", sort: null, groupBy: "month", contentSearch: false } },
];
let layout: LibraryLayoutDto = { sidebarWidth: 200, sidebarFolded: false, folderTreeWidth: 240, folderTreeHidden: false };
const diskFolders = [...FIXTURE_DIRECTORIES, "Archive"];
const KIND_LABELS = { pdf: "PDF", image: "Image", office: "Office", other: "Other" } as const;
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const lower = (text: string) => text.toLowerCase();
const byText = (first: string, second: string) => (first < second ? -1 : first > second ? 1 : 0);
const joinPath = (parent: string, name: string) => [parent, name].filter(Boolean).join("/");

function parentOf(path: string): string {
  const slash = path.lastIndexOf("/");
  return slash < 0 ? "" : path.slice(0, slash);
}

function inside(path: string, folder: string): boolean {
  return folder === "" || path.startsWith(`${folder}/`);
}

function category(id: string | null) {
  return FIXTURE_CATEGORIES.find((entry) => entry.id === id);
}

/** The category and its parents, innermost first. */
function categoryChain(id: string | null): string[] {
  const chain: string[] = [];
  for (let current = category(id); current && !chain.includes(current.id); current = category(current.parentId)) chain.push(current.id);
  return chain;
}

function categoryPath(id: string): string {
  const names = categoryChain(id).map((entry) => category(entry)?.name ?? "").reverse();
  return names.length ? names.join(" › ") : "Unknown category";
}

function yearOf(document: FixtureDocument): number {
  return Number((document.documentDate ?? document.addedAt).slice(0, 4));
}

function monthStart(): string {
  const now = new Date();
  return new Date(now.getFullYear(), now.getMonth(), 1).toISOString();
}

function message(text: string, isError = true) {
  void emit("message-event", { message: { text, isError } });
}

function baseMatches(view: LibraryViewDto, document: FixtureDocument): boolean {
  switch (view.kind) {
    case "folder":
      return inside(document.path, view.path);
    case "inbox":
      return inside(document.path, "Inbox");
    case "needsReview":
      return document.suggestion;
    case "recent":
      return document.addedAt >= monthStart();
    case "folders":
      return false;
    case "smartFolder": {
      const search = saved.find((entry) => entry.id === view.id);
      return search !== undefined && shown(search.params, true).some((other) => other.id === document.id);
    }
  }
}

function filterMatches(filter: Filter, document: FixtureDocument): boolean {
  switch (filter.kind) {
    case "category":
      return filter.categoryId === null ? document.categoryId === null : categoryChain(document.categoryId).includes(filter.categoryId);
    case "tag":
      return document.tags.some((tag) => lower(tag) === lower(filter.tag));
    case "year":
      return filter.andEarlier ? yearOf(document) <= filter.year : yearOf(document) === filter.year;
    case "correspondent":
      return lower(document.correspondent?.trim() ?? "") === lower(filter.name.trim());
    case "type":
      return document.kind === filter.documentKind;
    case "folder":
      return inside(document.path, filter.path);
  }
}

function textMatches(query: string, document: FixtureDocument): boolean {
  const wanted = lower(query.trim());
  if (!wanted) return true;
  return [document.path, document.title, document.correspondent, document.note, ...document.tags].some((text) => text && lower(text).includes(wanted));
}

function shown(params: Params, withFilters: boolean): FixtureDocument[] {
  const documents = FIXTURE_DOCUMENTS.filter(
    (document) =>
      baseMatches(params.view, document) &&
      (!withFilters || (params.filters.every((filter) => filterMatches(filter, document)) && textMatches(params.query, document))),
  );
  return params.view.kind === "recent" ? documents.sort((a, b) => byText(b.addedAt, a.addedAt)) : documents.sort((a, b) => byText(lower(a.path), lower(b.path)));
}

function rowOf(document: FixtureDocument): Row {
  return {
    id: document.id,
    path: document.path,
    fileName: fileName(document.path),
    folder: parentOf(document.path) || null,
    title: document.title,
    kind: document.kind,
    categoryName: category(document.categoryId)?.name ?? null,
    correspondent: document.correspondent,
    documentDate: document.documentDate,
    tags: [...document.tags].sort((a, b) => byText(lower(a), lower(b))),
    amount: document.tags.includes("invoice") ? `${(document.size / 1000).toFixed(2)} EUR` : null,
    dueDate: document.tags.includes("urgent") ? document.documentDate : null,
  };
}

function sorted(rows: Row[], params: Params): Row[] {
  const sort = params.sort;
  if (!sort) return rows;
  const value = (row: Row): string | null =>
    ({
      name: lower(row.title ?? row.fileName),
      category: row.categoryName && lower(row.categoryName),
      correspondent: row.correspondent && lower(row.correspondent),
      date: row.documentDate,
      amount: row.amount && row.amount.padStart(16, "0"),
    })[sort.column];
  const withValue = rows.filter((row) => value(row) !== null).sort((a, b) => byText(value(a) ?? "", value(b) ?? "") * (sort.descending ? -1 : 1));
  return [...withValue, ...rows.filter((row) => value(row) === null)];
}

function groups(rows: Row[], params: Params): LibraryStateDto_Serialize["groups"] {
  const keyed = new Map<string, { title: string; rows: Row[] }>();
  const missing: Row[] = [];
  for (const row of rows) {
    const document = FIXTURE_DOCUMENTS.find((entry) => entry.id === row.id);
    if (!document) continue;
    let key: string | null;
    let title: string;
    if (params.groupBy === "month") {
      const date = document.documentDate ?? document.addedAt.slice(0, 10);
      key = date.slice(0, 7);
      title = `${MONTHS[Number(date.slice(5, 7)) - 1]} ${date.slice(0, 4)}`;
    } else if (params.groupBy === "category") {
      const top = categoryChain(document.categoryId).at(-1);
      title = top ? (category(top)?.name ?? "") : "";
      key = top ? lower(title) : null;
    } else {
      title = document.correspondent?.trim() ?? "";
      key = document.correspondent === null ? null : lower(title);
    }
    if (key === null) missing.push(row);
    else keyed.set(key, { title: keyed.get(key)?.title ?? title, rows: [...(keyed.get(key)?.rows ?? []), row] });
  }
  const ordered = [...keyed.entries()].sort(([a], [b]) => (params.groupBy === "month" ? byText(b, a) : byText(a, b))).map(([, group]) => group);
  if (missing.length) ordered.push({ title: params.groupBy === "category" ? "Without a category" : "Unknown sender", rows: missing });
  return ordered;
}

function tokenOf(filter: Filter): Token {
  const keys: Record<Filter["kind"], string> = { category: "category", tag: "tag", year: "year", correspondent: "from", type: "type", folder: "folder" };
  let value: string;
  if (filter.kind === "category") value = filter.categoryId === null ? "Without a category" : categoryPath(filter.categoryId);
  else if (filter.kind === "tag") value = filter.tag;
  else if (filter.kind === "year") value = filter.andEarlier ? `Until ${filter.year}` : String(filter.year);
  else if (filter.kind === "correspondent") value = filter.name;
  else if (filter.kind === "type") value = KIND_LABELS[filter.documentKind];
  else value = filter.path;
  return { key: keys[filter.kind], value, filter };
}

function counted<T>(values: T[]): Map<T, number> {
  const counts = new Map<T, number>();
  values.forEach((value) => counts.set(value, (counts.get(value) ?? 0) + 1));
  return counts;
}

function byCount(options: Option[]): Option[] {
  return options.sort((a, b) => b.count - a.count || byText(lower(a.label), lower(b.label)));
}

type TreeCategory = { id: string; name: string; parentId: string | null; depth: number };

function categoryTree(parentId: string | null = null, depth = 0): TreeCategory[] {
  return FIXTURE_CATEGORIES.filter((entry) => entry.parentId === parentId)
    .sort((a, b) => byText(lower(a.name), lower(b.name)))
    .flatMap((entry) => [{ ...entry, depth }, ...categoryTree(entry.id, depth + 1)]);
}

function folderTree(documents: FixtureDocument[], extra: string[], expanded: string[] | null): LibraryFolderEntryDto[] {
  const folders = new Set<string>();
  const add = (folder: string) => {
    for (let current = folder; current; current = parentOf(current)) folders.add(current);
  };
  documents.forEach((document) => add(parentOf(document.path)));
  extra.forEach(add);
  const list = [...folders].sort((a, b) => byText(lower(a.replaceAll("/", "\0")), lower(b.replaceAll("/", "\0"))));
  const entries = list.map((path, index) => ({
    path,
    name: fileName(path),
    depth: path.split("/").length,
    documentCount: documents.filter((document) => inside(document.path, path)).length,
    hasChildren: list[index + 1]?.startsWith(`${path}/`) ?? false,
    expanded: expanded === null || expanded.includes(path),
  }));
  let hiddenBelow = Infinity;
  return entries.filter((entry) => {
    if (entry.depth > hiddenBelow) return false;
    hiddenBelow = entry.hasChildren && !entry.expanded ? entry.depth : Infinity;
    return true;
  });
}

function option(filter: Filter, label: string, count: number, depth = 0, missing = false): Option {
  return { filter, label, count, depth, missing };
}

function facets(documents: FixtureDocument[]): LibraryStateDto_Serialize["facets"] {
  const categoryCounts = counted(documents.flatMap((document) => categoryChain(document.categoryId)));
  const tagCounts = counted(documents.flatMap((document) => document.tags.map(lower)));
  const years = counted(documents.map(yearOf));
  const kinds = counted(documents.map((document) => document.kind));
  const correspondents = new Map<string, Option>();
  documents.forEach((document) => {
    const name = document.correspondent?.trim();
    if (!name) return;
    const existing = correspondents.get(lower(name));
    if (existing) existing.count++;
    else correspondents.set(lower(name), option({ kind: "correspondent", name }, name, 1));
  });
  const all: Record<LibraryFacetDto, Option[]> = {
    category: [
      ...categoryTree().map((entry) => option({ kind: "category", categoryId: entry.id }, entry.name, categoryCounts.get(entry.id) ?? 0, entry.depth)),
      option({ kind: "category", categoryId: null }, "Without a category", documents.filter((document) => document.categoryId === null).length, 0, true),
    ],
    tag: byCount(FIXTURE_TAGS.map((tag) => option({ kind: "tag", tag }, tag, tagCounts.get(lower(tag)) ?? 0))),
    year: [...years.keys()].sort((a, b) => b - a).map((year) => option({ kind: "year", year, andEarlier: false }, String(year), years.get(year) ?? 0)),
    correspondent: byCount([...correspondents.values()]),
    type: (["pdf", "image", "office", "other"] as const)
      .filter((kind) => kinds.has(kind))
      .map((kind) => option({ kind: "type", documentKind: kind }, KIND_LABELS[kind], kinds.get(kind) ?? 0)),
    folder: folderTree(documents, [], null).map((entry) => option({ kind: "folder", path: entry.path }, entry.name, entry.documentCount, entry.depth - 1)),
  };
  return (Object.keys(all) as LibraryFacetDto[]).map((facet) => ({ facet, options: all[facet] }));
}

function suggestions(query: string, documents: FixtureDocument[], active: Filter[]): Token[] {
  const wanted = lower(query.trim());
  if (!wanted) return [];
  const isActive = (filter: Filter) => active.some((existing) => JSON.stringify(existing) === JSON.stringify(filter));
  const tags: Filter[] = wanted.length >= 2 ? [...FIXTURE_TAGS].sort().filter((tag) => lower(tag).includes(wanted)).map((tag) => ({ kind: "tag", tag })) : [];
  const year: Filter[] = /^\d{4}$/.test(wanted) ? [{ kind: "year", year: Number(wanted), andEarlier: false }] : [];
  const names = [...new Map(documents.flatMap((document) => (document.correspondent ? [[lower(document.correspondent.trim()), document.correspondent.trim()] as const] : []))).values()];
  const people: Filter[] = wanted.length >= 3 ? names.sort().filter((name) => lower(name).includes(wanted)).map((name) => ({ kind: "correspondent", name })) : [];
  return [...tags, ...year, ...people].filter((filter) => !isActive(filter)).slice(0, 3).map(tokenOf);
}

function libraryState(params: Params): LibraryStateDto_Serialize {
  const inView = shown(params, false);
  const rows = sorted(shown(params, true).map(rowOf), params);
  const filtered = params.filters.length > 0 || params.query.trim() !== "";
  const view = params.view;
  const smart = view.kind === "smartFolder" ? saved.find((entry) => entry.id === view.id) : undefined;
  const titles: Record<LibraryViewDto["kind"], string> = {
    folder: "All documents",
    inbox: "Inbox",
    needsReview: "Needs review",
    recent: "Added this month",
    folders: "Folders",
    smartFolder: smart?.name ?? "Saved view",
  };
  const tops = new Map<string, FixtureDocument[]>();
  FIXTURE_DOCUMENTS.forEach((document) => {
    const top = categoryChain(document.categoryId).at(-1);
    if (top) tops.set(top, [...(tops.get(top) ?? []), document]);
  });
  return {
    title: titles[view.kind],
    counts: {
      all: FIXTURE_DOCUMENTS.length,
      inbox: FIXTURE_DOCUMENTS.filter((document) => inside(document.path, "Inbox")).length,
      needsReview: FIXTURE_DOCUMENTS.filter((document) => document.suggestion).length,
      addedThisMonth: FIXTURE_DOCUMENTS.filter((document) => document.addedAt >= monthStart()).length,
      folders: folderTree(FIXTURE_DOCUMENTS, [], null).length,
    },
    categories: categoryTree().map((entry) => ({ ...entry, documentCount: FIXTURE_DOCUMENTS.filter((document) => document.categoryId === entry.id).length })),
    tokens: [...(smart ? smart.params.filters.map((filter) => ({ ...tokenOf(filter), filter: null })) : []), ...params.filters.map(tokenOf)],
    suggestions: suggestions(params.query, inView, params.filters),
    facets: facets(inView),
    tiles:
      view.kind === "folder" && view.path === "" && !filtered
        ? [...tops.entries()]
            .map(([id, documents]) => ({
              categoryId: id,
              name: category(id)?.name ?? "",
              count: documents.length,
              kinds: documents.sort((a, b) => byText(a.path, b.path)).slice(0, 3).map((document) => document.kind),
            }))
            .sort((a, b) => byText(lower(a.name), lower(b.name)))
        : [],
    order: rows.map((row) => row.id),
    groups: groups(rows, params),
    smartFolders: saved.map((entry) => ({ id: entry.id, name: entry.name, count: shown(entry.params, true).length, invalid: false })),
    queryProblem: params.filters.some((filter) => filter.kind === "type")
      ? "A saved view or an export cannot keep the type filter yet"
      : view.kind === "needsReview"
        ? "Needs review cannot be saved or exported as a view"
        : null,
  };
}

function documentsText(count: number): string {
  return count === 1 ? "1 document" : `${count} documents`;
}

function plural(count: number, singular: string): string {
  return count === 1 ? `1 ${singular}` : `${count} ${singular}s`;
}

function requireIdle() {
  if (mockSession.state.operations.some((operation) => operation.lane === "library")) throw { code: "busy", message: "Wait until the running operation finishes" };
}

/** Changes the fixture as one library operation "{step}: {n documents}" that reports "{step}: {n} of {n …} done". */
function bulkOperation(step: string, ids: string[], change: (document: FixtureDocument) => void): boolean {
  if (ids.length === 0) return true;
  requireIdle();
  FIXTURE_DOCUMENTS.filter((document) => ids.includes(document.id)).forEach(change);
  mockOperation(`${step}: ${documentsText(ids.length)}`, "library", Math.min(ids.length, 6), `${step}: ${ids.length} of ${documentsText(ids.length)} done`);
  return true;
}

function libraryChange(label: string, done: string, change: () => void): boolean {
  requireIdle();
  change();
  mockOperation(label, "library", 2, done);
  return true;
}

function documentById(id: string): FixtureDocument | undefined {
  return FIXTURE_DOCUMENTS.find((document) => document.id === id);
}

function remove<T>(list: T[], keep: (entry: T) => boolean) {
  for (let index = list.length - 1; index >= 0; index--) if (!keep(list[index] as T)) list.splice(index, 1);
}

function moveFolder(from: string, to: string) {
  const moved = (path: string) => (path === from || path.startsWith(`${from}/`) ? to + path.slice(from.length) : path);
  FIXTURE_DOCUMENTS.forEach((document) => (document.path = moved(document.path)));
  diskFolders.forEach((folder, index) => (diskFolders[index] = moved(folder)));
}

function knownFolders(): string[] {
  return folderTree(FIXTURE_DOCUMENTS, diskFolders, null).map((entry) => entry.path);
}

function folderView(open: string, expanded: string[], query: string): LibraryFolderViewDto_Serialize {
  const all = knownFolders();
  const children = (parent: string) => all.filter((folder) => parentOf(folder) === parent);
  const crumbs = [{ path: "", name: "Library" }];
  open.split("/").filter(Boolean).forEach((segment, index, parts) => crumbs.push({ path: parts.slice(0, index + 1).join("/"), name: segment }));
  const wanted = lower(query.trim());
  const where = open ? `"${fileName(open)}"` : "the top level";
  const caption = (folder: string) => {
    const documents = FIXTURE_DOCUMENTS.filter((document) => inside(document.path, folder)).length;
    const folders = children(folder).length;
    return [documents ? plural(documents, "document") : "", folders ? plural(folders, "folder") : ""].filter(Boolean).join(" · ") || "Empty";
  };
  return {
    crumbs,
    tree: folderTree(FIXTURE_DOCUMENTS, diskFolders, expanded),
    libraryCount: FIXTURE_DOCUMENTS.length,
    tiles: children(open).map((path) => ({ path, name: fileName(path), caption: caption(path) })),
    documents: FIXTURE_DOCUMENTS.filter((document) => parentOf(document.path) === open)
      .filter((document) => !wanted || lower(document.title ?? "").includes(wanted) || lower(fileName(document.path)).includes(wanted))
      .sort((a, b) => byText(b.documentDate ?? "", a.documentDate ?? "") || byText(lower(fileName(a.path)), lower(fileName(b.path))))
      .map(rowOf),
    searchPlaceholder: `Search in ${where}…`,
    emptyText: `No documents directly in ${where}. Drop files here to add them.`,
  };
}

const views = {
  library_state: (args) => libraryState(args.params as Params),
  library_disk_folders: () => [...diskFolders],
  library_folder_view: (args) => {
    const params = args.params as { open: string; expanded: string[]; query: string };
    return folderView(params.open, params.expanded, params.query);
  },
  library_folder_tree: (args) => folderTree(FIXTURE_DOCUMENTS, diskFolders, args.expanded as string[]),
  library_proposed_folder_name: (args) => {
    const taken = new Set(knownFolders().filter((path) => parentOf(path) === args.parent).map((path) => lower(fileName(path))));
    let number = 1;
    while (taken.has(lower(number === 1 ? "New folder" : `New folder ${number}`))) number++;
    return number === 1 ? "New folder" : `New folder ${number}`;
  },
  library_layout: () => layout,
  library_set_layout: (args) => {
    layout = args.layout as LibraryLayoutDto;
    return null;
  },
} satisfies Record<string, Handler>;

const bulk = {
  library_bulk_assign_category: (args) => {
    const categoryId = args.categoryId as string | null;
    const ids = (args.documentIds as string[]).filter((id) => documentById(id)?.categoryId !== categoryId);
    return bulkOperation(categoryId ? `Filing under ${category(categoryId)?.name}` : "Removing the category", ids, (document) => (document.categoryId = categoryId));
  },
  library_bulk_add_tag: (args) => {
    const tag = (args.tag as string).trim();
    if (!tag || tag.length > 60) {
      message("A tag needs a name");
      return false;
    }
    if (!FIXTURE_TAGS.some((known) => lower(known) === lower(tag))) FIXTURE_TAGS.push(tag);
    const ids = (args.documentIds as string[]).filter((id) => !documentById(id)?.tags.some((known) => lower(known) === lower(tag)));
    return bulkOperation(`Tagging with ${tag}`, ids, (document) => document.tags.push(tag));
  },
  library_bulk_move: (args) => {
    const target = args.target as string;
    const ids = (args.documentIds as string[]).filter((id) => parentOf(documentById(id)?.path ?? "") !== target);
    return bulkOperation(`Moving to ${target || "the top level"}`, ids, (document) => (document.path = joinPath(target, fileName(document.path))));
  },
  library_bulk_trash: (args) => {
    const ids = args.documentIds as string[];
    if (ids.length === 0) return true;
    requireIdle();
    remove(FIXTURE_DOCUMENTS, (document) => !ids.includes(document.id));
    mockOperation(`Moving to the trash: ${documentsText(ids.length)}`, "library", 2, `Moving to the trash: ${ids.length} of ${documentsText(ids.length)} done`);
    return true;
  },
  library_bulk_tag_with_ai: (args) => {
    const count = (args.documentIds as string[]).length;
    mockOperation(`Tagging ${documentsText(count)} with AI`, "ai", count, `The AI suggested tags for ${count} of ${count} document(s); review them under Needs review`, true);
    return true;
  },
} satisfies Record<string, Handler>;

const taxonomy = {
  library_add_category: (args) => {
    const name = (args.name as string).trim();
    if (!name) return message("The name is empty"), false;
    return libraryChange("Saving the catalog", "Saved", () => FIXTURE_CATEGORIES.push({ id: `c-${Date.now()}`, name, parentId: args.parentId as string | null }));
  },
  library_rename_category: (args) =>
    libraryChange("Saving the catalog", "Saved", () => {
      const entry = category(args.categoryId as string);
      if (entry) entry.name = (args.name as string).trim();
    }),
  library_remove_category: (args) => libraryChange("Saving the catalog", "Saved", () => remove(FIXTURE_CATEGORIES, (entry) => entry.id !== args.categoryId)),
  library_rename_tag: (args) =>
    libraryChange("Saving the catalog", "Saved", () => {
      const from = lower(args.tag as string);
      const to = (args.name as string).trim();
      FIXTURE_TAGS.forEach((tag, index) => lower(tag) === from && (FIXTURE_TAGS[index] = to));
      FIXTURE_DOCUMENTS.forEach((document) => (document.tags = document.tags.map((tag) => (lower(tag) === from ? to : tag))));
    }),
  library_remove_tag_request: (args) => {
    const tag = args.tag as string;
    const tagged = FIXTURE_DOCUMENTS.filter((document) => document.tags.some((known) => lower(known) === lower(tag))).map((document) => document.path);
    return {
      title: `Remove tag "${tag}"`,
      message: `The tag is removed from the catalog and from ${tagged.length} document(s). The files stay as they are.`,
      affected: tagged,
      confirmLabel: "Remove tag",
    };
  },
  library_remove_tag: (args) =>
    libraryChange("Saving the catalog", "Saved", () => {
      const tag = lower(args.tag as string);
      remove(FIXTURE_TAGS, (known) => lower(known) !== tag);
      FIXTURE_DOCUMENTS.forEach((document) => (document.tags = document.tags.filter((known) => lower(known) !== tag)));
    }),
} satisfies Record<string, Handler>;

const savedViews = {
  library_save_view: (args) => {
    const name = (args.name as string).trim();
    return libraryChange("Saving the saved views", `Saved ${name}`, () => saved.push({ id: `s-${Date.now()}`, name, params: args.params as Params }));
  },
  library_replace_saved_view: (args) => {
    const entry = saved.find((search) => search.id === args.id);
    return !!entry && libraryChange("Saving the saved views", `Saved ${entry.name}`, () => (entry.params = args.params as Params));
  },
  library_rename_saved_view: (args) => {
    const entry = saved.find((search) => search.id === args.id);
    const name = (args.name as string).trim();
    return !!entry && libraryChange("Saving the saved views", `Saved ${name}`, () => (entry.name = name));
  },
  library_delete_saved_view: (args) => {
    const entry = saved.find((search) => search.id === args.id);
    return !!entry && libraryChange("Saving the saved views", `Deleted ${entry.name}`, () => remove(saved, (search) => search !== entry));
  },
  library_add_standard_saved_views: () =>
    libraryChange("Saving the saved views", "Added the standard saved views", () => {
      const year = new Date().getFullYear();
      const presets: [string, Filter[]][] = [
        ["Untagged", []],
        ["Uncategorized", [{ kind: "category", categoryId: null }]],
        [`Tax ${year}`, [{ kind: "year", year, andEarlier: false }]],
        ["Due in 30 days", []],
      ];
      presets
        .filter(([name]) => !saved.some((search) => lower(search.name) === lower(name)))
        .forEach(([name, filters]) => saved.push({ id: `s-${name}`, name, params: { view: ALL, filters, query: "", sort: null, groupBy: "month", contentSearch: false } }));
    }),
} satisfies Record<string, Handler>;

const folders = {
  library_create_folder: (args) => {
    const name = (args.name as string).trim().replaceAll("/", "-");
    const path = joinPath(args.parent as string, name);
    return !!name && libraryChange(`Creating folder ${path}`, `Created "${name}"`, () => diskFolders.push(path));
  },
  library_rename_folder: (args) => {
    const from = args.path as string;
    const name = (args.name as string).trim().replaceAll("/", "-");
    if (!name || name === fileName(from)) return false;
    return libraryChange(`Moving folder ${from}`, `Renamed to "${name}" in every library folder`, () => moveFolder(from, joinPath(parentOf(from), name)));
  },
  library_move_folder_into: (args) => {
    const from = args.path as string;
    const folder = args.folder as string;
    if (folder === from || folder.startsWith(`${from}/`)) return message("A folder cannot be moved into itself or one of its subfolders"), false;
    const to = joinPath(folder, fileName(from));
    return to !== from && libraryChange(`Moving folder ${from}`, `Moved "${fileName(from)}" to ${folder || "the top level"}`, () => moveFolder(from, to));
  },
  library_folder_delete_request: (args) => {
    const path = args.path as string;
    const documents = FIXTURE_DOCUMENTS.filter((document) => inside(document.path, path)).map((document) => document.path);
    const subfolders = knownFolders().filter((folder) => folder.startsWith(`${path}/`)).length;
    const parts = [documents.length ? plural(documents.length, "document") : "", subfolders ? plural(subfolders, "subfolder") : ""].filter(Boolean);
    const text = parts.length
      ? `${parts.join(" and ")} inside ${documents.length + subfolders === 1 ? "goes" : "go"} to the trash, in every library folder.`
      : "The folder is empty. It is removed from every library folder.";
    return { title: `Delete "${fileName(path)}"?`, message: text, affected: documents, confirmLabel: "Delete folder" };
  },
  library_trash_folder: (args) => {
    const path = args.path as string;
    const done = (args.listedFiles as number) > 0 ? `Deleted "${fileName(path)}"; its documents are in the trash` : `Deleted "${fileName(path)}"`;
    return libraryChange(`Deleting folder ${path}`, done, () => {
      remove(FIXTURE_DOCUMENTS, (document) => !inside(document.path, path));
      remove(diskFolders, (folder) => folder !== path && !folder.startsWith(`${path}/`));
    });
  },
  library_add_files: (args) => {
    const count = (args.paths as string[]).length;
    mockOperation(`Adding ${count} file(s)`, "library", count, `Added ${count} of ${count} file(s)`);
    return null;
  },
  library_export: (args) => libraryChange(`Exporting ${args.scopeLabel as string}`, `Exported 3 documents to ${args.directory as string}/${args.name as string}`, () => undefined),
} satisfies Record<string, Handler>;

/** Mocked commands of features::library. */
export const libraryHandlers: Record<string, Handler> = { ...views, ...bulk, ...taxonomy, ...savedViews, ...folders };
