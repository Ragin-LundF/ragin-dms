import SEARCH_OFF from "@material-design-icons/svg/round/search_off.svg?raw";
import { useRef } from "react";
import { DropOverlay, EmptyState, Button } from "../../../shared/components";
import { useFileDrop } from "../interaction/file-drop";
import { addFiles } from "../state/actions";
import type { LibraryState } from "../state/queries";
import { addFilter, clearFilters, filteredFolder, isFiltered, nextSort, viewStore, type ViewState } from "../state/view-state";
import { CategoryTiles } from "./category-tiles";
import { DocumentGrid, type DocumentCallbacks } from "./document-grid";
import { DocumentList } from "./document-list";
import { SelectionBar } from "./selection-bar";

/**
 * The documents of the view (Kotlin `DocumentArea`): the category tiles on "All documents" without filters, then the
 * grouped grid or list, and the bulk bar while documents are ticked. Files dropped here go to the filtered folder.
 */
export function DocumentArea({ view, state, callbacks, ticked, busyReason, aiBusyReason, aiEnabled }: {
  view: ViewState;
  state: LibraryState;
  callbacks: DocumentCallbacks;
  /** The ticked documents that are shown. */
  ticked: number;
  busyReason: string | null;
  aiBusyReason: string | null;
  aiEnabled: boolean;
}) {
  const zone = useRef<HTMLDivElement>(null);
  const dropping = useFileDrop(zone, (paths) => addFiles(paths));
  const folder = filteredFolder(view);
  return (
    <div className="column gap-m" style={{ flex: 1, minHeight: 0 }}>
      {state.tiles.length > 0 && <CategoryTiles tiles={state.tiles} onPick={(categoryId) => addFilter({ kind: "category", categoryId })} />}
      <div ref={zone} className="lib-area">
        <Documents view={view} state={state} callbacks={callbacks} />
        {dropping && <DropOverlay label={`The files go to ${folder || "the top level"} in every library folder`} />}
        {ticked > 0 && <SelectionBar state={state} count={ticked} busyReason={busyReason} aiBusyReason={aiBusyReason} aiEnabled={aiEnabled} />}
      </div>
    </div>
  );
}

function Documents({ view, state, callbacks }: { view: ViewState; state: LibraryState; callbacks: DocumentCallbacks }) {
  if (state.order.length === 0) {
    return isFiltered(view) ? (
      <EmptyState
        icon={SEARCH_OFF}
        title="No documents match"
        message="Remove a filter or change the search to see more."
        action={<Button onClick={clearFilters}>Clear filters</Button>}
        style={{ height: "100%" }}
      />
    ) : (
      <EmptyState
        icon={SEARCH_OFF}
        title="No documents here yet"
        message="Drop files here, or use Add files. They are copied into every library folder."
        style={{ height: "100%" }}
      />
    );
  }
  if (view.layout === "list") {
    return (
      <DocumentList
        groups={state.groups}
        sort={view.sort}
        onSort={(column) => viewStore.set((current) => ({ ...current, sort: nextSort(current.sort, column) }))}
        {...callbacks}
      />
    );
  }
  return <DocumentGrid groups={state.groups} {...callbacks} />;
}
