import { defaultRangeExtractor, useVirtualizer, type Range } from "@tanstack/react-virtual";
import { useCallback, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { DocumentRow } from "../state/queries";

/** Space between cards, as `--space-m`. */
const GAP = 12;
const CARD_MIN_WIDTH = 132;
/** Card padding, title and meta line below the 3:4 page. */
const CARD_TEXT_HEIGHT = 64;
const HEADER_HEIGHT = 52;
const LIST_ROW_HEIGHT = 56;

type Entry = { kind: "header"; title: string; count: number } | { kind: "row"; rows: DocumentRow[] };

/**
 * The groups of documents in one scrolling area where only the visible rows are in the DOM, like Kotlin's lazy grid
 * and list (Hoard Deck's virtual grid). `grid` lays the documents out in as many columns of at least 132 px as fit;
 * otherwise one per row. Group headers stay at the top while their group scrolls.
 */
export function VirtualSections({ groups, headers, grid, renderItem, rowClassName }: {
  groups: { title: string; rows: DocumentRow[] }[];
  headers: boolean;
  grid: boolean;
  renderItem: (row: DocumentRow) => ReactNode;
  rowClassName: string;
}) {
  const scroll = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = scroll.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => setWidth(entry?.contentRect.width ?? 0));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  const columns = grid ? Math.max(1, Math.floor((width + GAP) / (CARD_MIN_WIDTH + GAP))) : 1;
  const entries = useMemo(() => flatten(groups, headers, columns), [groups, headers, columns]);
  const headerIndexes = useMemo(() => entries.flatMap((entry, index) => (entry.kind === "header" ? [index] : [])), [entries]);
  const cardWidth = width > 0 ? (width - GAP * (columns - 1)) / columns : CARD_MIN_WIDTH;
  const rowHeight = grid ? Math.round((cardWidth * 4) / 3 + CARD_TEXT_HEIGHT + GAP) : LIST_ROW_HEIGHT;
  const activeHeader = useRef(0);
  const rangeExtractor = useCallback(
    (range: Range) => {
      activeHeader.current = [...headerIndexes].reverse().find((index) => range.startIndex >= index) ?? -1;
      const visible = defaultRangeExtractor(range);
      return activeHeader.current >= 0 && !visible.includes(activeHeader.current) ? [activeHeader.current, ...visible] : visible;
    },
    [headerIndexes],
  );
  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => scroll.current,
    estimateSize: (index) => (entries[index]?.kind === "header" ? HEADER_HEIGHT : rowHeight),
    getItemKey: (index) => keyOf(entries[index], index),
    rangeExtractor,
    overscan: 4,
  });
  useLayoutEffect(() => virtualizer.measure(), [rowHeight, columns, virtualizer]);
  return (
    <div ref={scroll} className="lib-scroll">
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualizer.getVirtualItems().map((item) => {
          const entry = entries[item.index];
          if (!entry) return null;
          const sticky = entry.kind === "header" && item.index === activeHeader.current;
          const place = sticky
            ? { position: "sticky" as const, top: 0, zIndex: 2 }
            : { position: "absolute" as const, top: 0, transform: `translateY(${item.start}px)` };
          return (
            <div key={item.key} data-index={item.index} ref={virtualizer.measureElement} style={{ ...place, left: 0, right: 0, width: "100%" }}>
              {entry.kind === "header" ? (
                <h3 className="lib-group-header">
                  <span className="title-small">{entry.title}</span>
                  <span className="label-medium muted">· {entry.count}</span>
                </h3>
              ) : (
                <div className={rowClassName} style={grid ? { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`, paddingBottom: GAP } : undefined}>
                  {entry.rows.map((row) => <div key={row.id} style={{ minWidth: 0 }}>{renderItem(row)}</div>)}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function flatten(groups: { title: string; rows: DocumentRow[] }[], headers: boolean, columns: number): Entry[] {
  const entries: Entry[] = [];
  for (const group of groups) {
    if (headers) entries.push({ kind: "header", title: group.title, count: group.rows.length });
    for (let start = 0; start < group.rows.length; start += columns) {
      entries.push({ kind: "row", rows: group.rows.slice(start, start + columns) });
    }
  }
  return entries;
}

function keyOf(entry: Entry | undefined, index: number): string {
  if (!entry) return String(index);
  return entry.kind === "header" ? `header:${entry.title}` : `row:${entry.rows[0]?.id ?? index}`;
}
