import type { LibrarySortColumnDto, LibrarySortDto } from "../../../ipc";
import { Icon, KIND_STYLES, LabelChip, PageThumbnail } from "../../../shared/components";
import type { DocumentRow } from "../state/queries";
import { VirtualSections } from "./virtual-sections";
import { SelectionCircle, clickOf, pressDocument, titleOf, type DocumentCallbacks } from "./document-grid";

/** The list's column weights, shared by header and rows so they line up (Kotlin `DocumentColumns`). */
const WEIGHTS = { name: 3, category: 1.3, tags: 1.6, correspondent: 1.4, date: 0.9, amount: 0.9 };

const COLUMNS: { text: string; weight: number; column: LibrarySortColumnDto | null; end?: boolean }[] = [
  { text: "Name", weight: WEIGHTS.name, column: "name" },
  { text: "Category", weight: WEIGHTS.category, column: "category" },
  { text: "Tags", weight: WEIGHTS.tags, column: null },
  { text: "From", weight: WEIGHTS.correspondent, column: "correspondent" },
  { text: "Date", weight: WEIGHTS.date, column: "date" },
  { text: "Amount", weight: WEIGHTS.amount, column: "amount", end: true },
];

const cell = (weight: number) => ({ flex: `${weight} 1 0` });

/**
 * The documents as rows under a header that stays in place, grouped like the grid (Kotlin `DocumentList`); a click
 * on a column title sorts by it.
 */
export function DocumentList({ groups, sort, onSort, ...callbacks }: {
  groups: { title: string; rows: DocumentRow[] }[];
  sort: LibrarySortDto | null;
  onSort: (column: LibrarySortColumnDto) => void;
} & DocumentCallbacks) {
  const selecting = callbacks.selected.size > 0;
  return (
    <div className="column" style={{ height: "100%" }}>
      <div className="lib-list-header">
        <span style={{ width: 20, flex: "none" }} />
        <span style={{ width: 22, flex: "none" }} />
        {COLUMNS.map(({ text, weight, column, end }) => {
          const arrow = column && sort?.column === column ? (sort.descending ? " ↓" : " ↑") : "";
          return (
            <span
              key={text}
              className={column ? "lib-cell label-medium sortable" : "lib-cell label-medium"}
              style={{ ...cell(weight), textAlign: end ? "end" : "start" }}
              role={column ? "button" : undefined}
              aria-label={column ? `Sort by ${text}` : undefined}
              onClick={column ? () => onSort(column) : undefined}
            >
              {text + arrow}
            </span>
          );
        })}
      </div>
      <div className="divider" />
      <div className="grow" style={{ flex: 1, minHeight: 0 }}>
        <VirtualSections
          groups={groups}
          headers
          grid={false}
          rowClassName="lib-rows"
          renderItem={(row) => <DocumentListRow row={row} selecting={selecting} {...callbacks} />}
        />
      </div>
    </div>
  );
}

/** One document as a row: circle, thumbnail, name, category, tags, sender, date and amount (Kotlin `DocumentListRow`). */
function DocumentListRow({ row, selecting, selected, inspected, folderId, onClick, onToggle }: { row: DocumentRow; selecting: boolean } & DocumentCallbacks) {
  const ticked = selected.has(row.id);
  const title = titleOf(row);
  return (
    <div
      role="button"
      tabIndex={-1}
      aria-label={`Open ${title}`}
      className={ticked || inspected === row.id ? "lib-row selected" : "lib-row"}
      onPointerDown={(event) => pressDocument(event, row)}
      onClick={(event) => onClick(row.id, clickOf(event))}
    >
      <SelectionCircle title={title} selected={ticked} visible={selecting} onToggle={() => onToggle(row.id)} />
      <RowThumbnail row={row} folderId={folderId} />
      <div className="column" style={{ ...cell(WEIGHTS.name), minWidth: 0 }}>
        <span className="body-medium ellipsis">{title}</span>
        <span className="body-small muted ellipsis">{row.title === null ? (row.folder ?? "Top level") : row.fileName}</span>
      </div>
      <div className="lib-cell" style={cell(WEIGHTS.category)}>
        {row.categoryName === null ? <span className="body-small muted" style={{ fontStyle: "italic" }}>none</span> : <LabelChip text={row.categoryName} color="var(--tertiary)" />}
      </div>
      <div className="lib-tags" style={cell(WEIGHTS.tags)}>
        {row.tags.slice(0, 2).map((tag) => <LabelChip key={tag} text={tag} color="var(--secondary)" />)}
        {row.tags.length > 2 && <span className="label-medium">+{row.tags.length - 2}</span>}
      </div>
      <span className="lib-cell body-small muted" style={cell(WEIGHTS.correspondent)}>{row.correspondent ?? ""}</span>
      <span className="lib-cell body-small" style={cell(WEIGHTS.date)}>{row.documentDate ?? ""}</span>
      <span className="lib-cell body-small" style={{ ...cell(WEIGHTS.amount), textAlign: "end" }}>{row.amount ?? ""}</span>
    </div>
  );
}

function RowThumbnail({ row, folderId }: { row: DocumentRow; folderId: string | null }) {
  const style = KIND_STYLES[row.kind];
  const tile = (
    <div className="lib-thumb" style={{ background: `color-mix(in srgb, ${style.color} 14%, transparent)`, color: style.color }}>
      <Icon svg={style.icon} label={style.label} />
    </div>
  );
  if (folderId === null) return tile;
  return (
    <div className="lib-thumb">
      <PageThumbnail folderId={folderId} path={row.path} fallback={tile} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
