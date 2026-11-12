import type {
  LibraryFilterDto_Serialize,
  LibraryGroupKeyDto,
  LibraryParamsDto_Serialize,
  LibrarySortColumnDto,
  LibrarySortDto,
  LibraryViewDto,
} from "../../../ipc";
import {createStore} from "../../../shared/store";
import type {LibrarySelection} from "../navigation";

export type FacetFilter = LibraryFilterDto_Serialize;
export type LibraryParams = LibraryParamsDto_Serialize;
export type Layout = "grid" | "list";
export type SidebarSection = "library" | "savedViews";

/**
 * What the user chose on the library screen (Kotlin `LibraryViewStateModel`): the sidebar's view, the facet filters
 * (at most one per facet, in the order added), the search text and how the documents are laid out. Nothing of it is
 * stored across launches.
 */
export type ViewState = {
  view: LibraryViewDto;
  filters: FacetFilter[];
  query: string;
  layout: Layout;
  groupBy: LibraryGroupKeyDto;
  sort: LibrarySortDto | null;
  /** Whether the search reads the documents' text too (Kotlin `ContentSearch`, on by default). */
  contentSearch: boolean;
  collapsed: SidebarSection[];
};

export const ALL_DOCUMENTS: LibraryViewDto = { kind: "folder", path: "" };

export const viewStore = createStore<ViewState>({
  view: ALL_DOCUMENTS,
  filters: [],
  query: "",
  layout: "grid",
  groupBy: "month",
  sort: null,
  contentSearch: true,
  collapsed: [],
});

/** Grows each time something asks the search field to take the focus ("/"). */
export const searchFocus = createStore(0);

export function paramsOf(state: ViewState): LibraryParams {
  return { view: state.view, filters: state.filters, query: state.query, sort: state.sort, groupBy: state.groupBy, contentSearch: state.contentSearch };
}

/** `true` while a facet filter or search text narrows the documents. */
export function isFiltered(state: Pick<ViewState, "filters" | "query">): boolean {
  return state.filters.length > 0 || state.query.trim().length > 0;
}

export function sameView(first: LibraryViewDto, second: LibraryViewDto): boolean {
  return JSON.stringify(first) === JSON.stringify(second);
}

export function sameFilter(first: FacetFilter, second: FacetFilter): boolean {
  if (first.kind !== second.kind) return false;
  switch (first.kind) {
    case "category":
      return first.categoryId === (second as typeof first).categoryId;
    case "tag":
      return first.tag.toLowerCase() === (second as typeof first).tag.toLowerCase();
    case "year":
      return first.year === (second as typeof first).year && first.andEarlier === (second as typeof first).andEarlier;
    case "correspondent":
      return first.name === (second as typeof first).name;
    case "type":
      return first.documentKind === (second as typeof first).documentKind;
    case "payment":
      return first.status === (second as typeof first).status;
    case "folder":
      return first.path === (second as typeof first).path;
    case "tax":
      return first.year === (second as typeof first).year && first.area === (second as typeof first).area;
    case "dates":
      return first.from === (second as typeof first).from && first.to === (second as typeof first).to;
    case "field": {
      const other = second as typeof first;
      return first.key === other.key && first.comparison === other.comparison && first.value === other.value;
    }
  }
}

/** `filters` with `filter` at the end, in place of the filter its facet had. */
export function withFilter(filters: FacetFilter[], filter: FacetFilter): FacetFilter[] {
  return [...filters.filter((existing) => existing.kind !== filter.kind), filter];
}

/** The sort after a click on `column`: the same column turns around, another one starts ascending. */
export function nextSort(current: LibrarySortDto | null, column: LibrarySortColumnDto): LibrarySortDto {
  return current?.column === column ? { column, descending: !current.descending } : { column, descending: false };
}

/**
 * The view that opens `selection`: a sidebar view without filters, or "All documents" filtered by the category, tag
 * or folder below the root it names. The search text is cleared.
 */
export function opening(state: ViewState, selection: LibrarySelection): ViewState {
  const opened = (view: LibraryViewDto, filter: FacetFilter | null): ViewState => ({ ...state, view, filters: filter ? [filter] : [], query: "" });
  switch (selection.kind) {
    case "folder":
      return opened(ALL_DOCUMENTS, selection.path ? { kind: "folder", path: selection.path } : null);
    case "category":
      return opened(ALL_DOCUMENTS, { kind: "category", categoryId: selection.categoryId });
    case "uncategorized":
      return opened(ALL_DOCUMENTS, { kind: "category", categoryId: null });
    case "tag":
      return opened(ALL_DOCUMENTS, { kind: "tag", tag: selection.tag });
    default:
      return opened(selection, null);
  }
}

/** The folder the library filters by, where "Add files" puts the files; the top level without one. */
export function filteredFolder(state: ViewState): string {
  const folder = state.filters.find((filter) => filter.kind === "folder");
  return folder?.kind === "folder" ? folder.path : "";
}

/** Applies `filter` in place of its facet's filter; a suggestion taken from the search clears the search text. */
export function addFilter(filter: FacetFilter, clearQuery = false) {
  viewStore.set((state) => ({ ...state, filters: withFilter(state.filters, filter), query: clearQuery ? "" : state.query }));
}

export function removeFilter(filter: FacetFilter) {
  viewStore.set((state) => ({ ...state, filters: state.filters.filter((existing) => !sameFilter(existing, filter)) }));
}

/** Removes every facet filter and the search text. */
export function clearFilters() {
  viewStore.set((state) => ({ ...state, filters: [], query: "" }));
}
