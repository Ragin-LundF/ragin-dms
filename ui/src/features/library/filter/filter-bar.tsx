import ARCHIVE from "@material-design-icons/svg/round/archive.svg?raw";
import BOLT from "@material-design-icons/svg/round/bolt.svg?raw";
import FIND from "@material-design-icons/svg/round/find_in_page.svg?raw";
import GRID from "@material-design-icons/svg/round/grid_view.svg?raw";
import ROWS from "@material-design-icons/svg/round/table_rows.svg?raw";
import type {ReactNode} from "react";
import type {LibraryGroupKeyDto} from "../../../ipc";
import {ActionButtons, ActionMenu, Button, Icon} from "../../../shared/components";
import {openExport} from "../export/export-state";
import {savedViews} from "../state/actions";
import type {LibraryState} from "../state/queries";
import {
    addFilter,
    clearFilters,
    isFiltered,
    paramsOf,
    removeFilter,
    type ViewState,
    viewStore
} from "../state/view-state";
import {FACET_ORDER, FacetChip} from "./facet-chip";
import {TokenSearchField} from "./token-search-field";

const GROUPS: { key: LibraryGroupKeyDto; label: string }[] = [
  { key: "month", label: "By month" },
  { key: "category", label: "By category" },
  { key: "sender", label: "By sender" },
];

function change(patch: Partial<ViewState>) {
  viewStore.set((state) => ({ ...state, ...patch }));
}

/**
 * Above the documents (Kotlin `FilterBar`): the search field with the active filters as tokens, below it the facet
 * chips, "Clear" and "Save view" while anything filters, and on the right grouping, grid/list and the ⋯ menu.
 */
export function FilterBar({ view, state, busyReason }: { view: ViewState; state: LibraryState | undefined; busyReason: string | null }) {
  const tokens = state?.tokens ?? [];
  const placeholder =
    tokens.length > 0 ? "Add words…" : view.contentSearch ? "Search text, names, tags… try “invoice” or “2025”" : "Search names, titles, correspondents and tags…";
  const problem = state?.queryProblem ?? null;
  return (
    <div className="column gap-m" style={{ flex: "none" }}>
      <TokenSearchField
        query={view.query}
        tokens={tokens}
        suggestions={state?.suggestions ?? []}
        placeholder={placeholder}
        onQuery={(query) => change({ query })}
        onRemove={removeFilter}
        onTake={(filter) => addFilter(filter, true)}
      />
      <div className="lib-facet-row">
        <div className="lib-facets">
          {FACET_ORDER.map((facet) => (
            <FacetChip key={facet} facet={facet} state={state} filters={view.filters} busyReason={busyReason} onPick={(filter) => addFilter(filter)} onRemove={removeFilter} />
          ))}
          {isFiltered(view) && (
            <>
              <Button variant="text" onClick={clearFilters}>Clear</Button>
              <ActionButtons
                buttons={[{ label: "Save view", icon: BOLT, style: "outlined", disabledReason: problem, onClick: () => void savedViews.saveCurrent() }]}
                enabled={busyReason === null}
                busyReason={busyReason}
              />
            </>
          )}
        </div>
        <Segments>
          {GROUPS.map((group) => (
            <Segment key={group.key} selected={view.groupBy === group.key} onClick={() => change({ groupBy: group.key })}>
              <span className="label-medium">{group.label}</span>
            </Segment>
          ))}
        </Segments>
        <Segments>
          <Segment selected={view.layout === "grid"} label="Grid" onClick={() => change({ layout: "grid" })}>
            <Icon svg={GRID} />
          </Segment>
          <Segment selected={view.layout === "list"} label="List" onClick={() => change({ layout: "list" })}>
            <Icon svg={ROWS} />
          </Segment>
        </Segments>
        <ActionMenu
          label="More library actions"
          enabled={busyReason === null}
          busyReason={busyReason}
          actions={[
            {
              label: "Export…",
              icon: ARCHIVE,
              disabledReason: problem ?? ((state?.order.length ?? 0) === 0 ? "Nothing to export" : null),
              onClick: () => openExport({ kind: "view", params: paramsOf(view) }, state?.title ?? "All documents"),
            },
            {
              label: view.contentSearch ? "Search names only" : "Search the documents' text too",
              icon: FIND,
              onClick: () => change({ contentSearch: !view.contentSearch }),
            },
          ]}
        />
      </div>
    </div>
  );
}

function Segments({ children }: { children: ReactNode }) {
  return <div className="lib-segments" role="tablist">{children}</div>;
}

function Segment({ selected, label, onClick, children }: { selected: boolean; label?: string; onClick: () => void; children: ReactNode }) {
  return (
    <button type="button" role="tab" aria-selected={selected} aria-label={label} className={selected ? "lib-segment selected" : "lib-segment"} onClick={onClick}>
      {children}
    </button>
  );
}
