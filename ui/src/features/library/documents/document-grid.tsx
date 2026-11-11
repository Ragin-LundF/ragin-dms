import CHECK from "@material-design-icons/svg/round/check.svg?raw";
import { Icon, KIND_STYLES, PageThumbnail } from "../../../shared/components";
import { beginPress } from "../interaction/drag";
import type { DocumentRow } from "../state/queries";
import { VirtualSections } from "./virtual-sections";

/** How a document was clicked: plainly, with ⌘/Ctrl to tick it, or with Shift to tick a range (Shift wins). */
export type DocumentClick = "plain" | "toggle" | "range";

export function clickOf(event: React.MouseEvent): DocumentClick {
  if (event.shiftKey) return "range";
  return event.metaKey || event.ctrlKey ? "toggle" : "plain";
}

export type DocumentCallbacks = {
  selected: ReadonlySet<string>;
  inspected: string | null;
  folderId: string | null;
  onClick: (id: string, click: DocumentClick) => void;
  onToggle: (id: string) => void;
};

export function titleOf(row: DocumentRow): string {
  return row.title ?? row.fileName;
}

/** Starts a drag of the document on a press that moves (a click still clicks). */
export function pressDocument(event: React.PointerEvent, row: DocumentRow) {
  beginPress(event, { kind: "document", id: row.id, title: titleOf(row), documentKind: row.kind });
}

/**
 * The documents as a grid of page previews, as many columns of at least 132 px as fit (Kotlin `DocumentGrid`), under
 * each group's sticky header unless `headers` is off. Scrolls by itself; only the visible rows are rendered.
 */
export function DocumentGrid({ groups, headers = true, ...callbacks }: { groups: { title: string; rows: DocumentRow[] }[]; headers?: boolean } & DocumentCallbacks) {
  const selecting = callbacks.selected.size > 0;
  return (
    <VirtualSections
      groups={groups}
      headers={headers}
      grid
      rowClassName="lib-grid"
      renderItem={(row) => <DocumentCard row={row} selecting={selecting} {...callbacks} />}
    />
  );
}

/**
 * One document of the grid (Kotlin `DocumentCard`): its first page at 3:4 with the due date and amount on it, then
 * title and "sender · category". The circle shows on hover and while anything is ticked; quick look's document has
 * a ring.
 */
function DocumentCard({ row, selecting, selected, inspected, folderId, onClick, onToggle }: { row: DocumentRow; selecting: boolean } & DocumentCallbacks) {
  const ticked = selected.has(row.id);
  const title = titleOf(row);
  const classes = ["lib-card", ticked ? "selected" : "", inspected === row.id ? "inspected" : ""].filter(Boolean).join(" ");
  return (
    <div
      role="button"
      tabIndex={-1}
      aria-label={`Open ${title}`}
      className={classes}
      onPointerDown={(event) => pressDocument(event, row)}
      onClick={(event) => onClick(row.id, clickOf(event))}
    >
      <PagePreview row={row} folderId={folderId} />
      <div className="column">
        <span className="body-medium ellipsis" style={{ fontWeight: 500 }}>{title}</span>
        <span className="body-small muted ellipsis">{[row.correspondent, row.categoryName].filter(Boolean).join(" · ")}</span>
      </div>
      <SelectionCircle title={title} selected={ticked} visible={selecting} onToggle={() => onToggle(row.id)} />
    </div>
  );
}

/** The first page as a sheet of paper; until it is there, and for files without one, a sheet in the kind's colour. */
function PagePreview({ row, folderId }: { row: DocumentRow; folderId: string | null }) {
  const blank = <BlankPage kind={row.kind} />;
  return (
    <div className="lib-page">
      {folderId === null ? blank : <PageThumbnail folderId={folderId} path={row.path} fallback={blank} />}
      <div className="lib-page-labels">
        {row.dueDate && <span className="lib-page-label warning label-small">due {row.dueDate}</span>}
        {row.amount && <span className="lib-page-label label-small">{row.amount}</span>}
      </div>
    </div>
  );
}

function BlankPage({ kind }: { kind: DocumentRow["kind"] }) {
  const style = KIND_STYLES[kind];
  return (
    <div className="lib-blank">
      <div className="lib-blank-band" style={{ background: style.color }} />
      <div className="lib-blank-icon" style={{ color: style.color }}>
        <Icon svg={style.icon} label={style.label} />
      </div>
    </div>
  );
}

/** The circle that ticks a document for the bulk bar; it shows on hover, while `visible`, or while ticked. */
export function SelectionCircle({ title, selected, visible, onToggle }: { title: string; selected: boolean; visible: boolean; onToggle: () => void }) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={selected}
      aria-label={`Select ${title}`}
      className={["lib-circle", selected ? "checked" : "", visible || selected ? "" : "hover-only"].filter(Boolean).join(" ")}
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => {
        event.stopPropagation();
        onToggle();
      }}
    >
      {selected && <Icon svg={CHECK} />}
    </button>
  );
}
