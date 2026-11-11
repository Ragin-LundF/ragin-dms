import {useRef} from "react";
import UPLOAD from "@material-design-icons/svg/round/cloud_upload.svg?raw";
import OPEN_FULL from "@material-design-icons/svg/round/open_in_full.svg?raw";
import {IconButton} from "../controls/buttons";
import {IconBadge} from "./cards";

/**
 * A divider between two panes that resizes the pane to its left when dragged sideways. `onResizeEnd` runs once
 * when the drag ends, so a caller can store the width then; a double-click can reset the pane.
 */
export function ResizeHandle({ onResize, onResizeEnd, onDoubleClick }: { onResize: (delta: number) => void; onResizeEnd?: () => void; onDoubleClick?: () => void }) {
  const last = useRef<number | null>(null);
  return (
    <div
      className="resize-handle"
      role="separator"
      aria-orientation="vertical"
      onDoubleClick={onDoubleClick}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        last.current = event.clientX;
      }}
      onPointerMove={(event) => {
        if (last.current === null) return;
        onResize(event.clientX - last.current);
        last.current = event.clientX;
      }}
      onPointerUp={() => {
        if (last.current === null) return;
        last.current = null;
        onResizeEnd?.();
      }}
    />
  );
}

/** The highlighted area that tells where dropped files will go. */
export function DropOverlay({ label }: { label: string }) {
  return (
    <div className="drop-overlay">
      <IconBadge icon={UPLOAD} tint="var(--primary)" size={72} />
      <span className="headline-small">Drop to add</span>
      <span className="body-large">{label}</span>
    </div>
  );
}

/** Opens the full view of a dashboard card. */
export function FullViewButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <IconButton icon={OPEN_FULL} label={label} tooltip onClick={onClick} />;
}
