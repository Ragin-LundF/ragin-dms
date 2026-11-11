import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import { IconBadge, KindBadge } from "../../../shared/components";
import { useStore } from "../../../shared/store";
import { dragStore, dropAction } from "../interaction/drag";

/**
 * The card that follows the pointer while a document or folder is dragged, saying what a drop would do (Kotlin
 * `DragOverlay` + `DragPreview`); a ticked document drags all `selected` documents along.
 */
export function DragPreview({ selected }: { selected: ReadonlySet<string> }) {
  const drag = useStore(dragStore);
  const item = drag.item;
  if (!item) return null;
  const count = item.kind === "document" && selected.has(item.id) ? selected.size : 1;
  const title = item.kind === "folder" ? item.path.slice(item.path.lastIndexOf("/") + 1) : count > 1 ? `${count} documents` : item.title;
  const action = drag.hovered ? dropAction(item, drag.hovered) : null;
  return (
    <div className="lib-drag-preview" style={{ left: drag.x + 16, top: drag.y + 12 }}>
      {item.kind === "document" ? <KindBadge kind={item.documentKind} size={32} /> : <IconBadge icon={FOLDER} tint="var(--primary)" size={32} />}
      <div className="column" style={{ minWidth: 0 }}>
        <span className="title-small ellipsis">{title}</span>
        <span className={action ? "body-small primary-text ellipsis" : "body-small muted ellipsis"}>{action ?? "Drop on a folder"}</span>
      </div>
    </div>
  );
}
