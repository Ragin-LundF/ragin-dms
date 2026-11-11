import { KIND_STYLES } from "../../../shared/components";
import type { LibraryState } from "../state/queries";

/** The top-level categories as tiles above "All documents" (Kotlin `CategoryTiles`); a click filters by one. */
export function CategoryTiles({ tiles, onPick }: { tiles: LibraryState["tiles"]; onPick: (categoryId: string) => void }) {
  return (
    <div className="lib-tiles">
      {tiles.map((tile) => (
        <button key={tile.categoryId} type="button" className="lib-tile" onClick={() => onPick(tile.categoryId)}>
          <div className="lib-stack">
            {tile.kinds.map((kind, index) => (
              <div
                key={index}
                className="lib-stack-page"
                style={{ left: 16 * index, top: 2 * (tile.kinds.length - 1 - index), transform: `rotate(${(index - 1) * 4}deg)` }}
              >
                <div style={{ background: KIND_STYLES[kind].color }} />
              </div>
            ))}
          </div>
          <div className="column">
            <span className="body-medium ellipsis" style={{ fontWeight: 600 }}>{tile.name}</span>
            <span className="body-small muted">{tile.count === 1 ? "1 document" : `${tile.count} documents`}</span>
          </div>
        </button>
      ))}
    </div>
  );
}
