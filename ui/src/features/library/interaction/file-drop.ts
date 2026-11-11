import {type RefObject, useEffect, useRef, useState} from "react";
import {getCurrentWebview} from "@tauri-apps/api/webview";
import {isTauri} from "../../../shell/tauri";

/**
 * Takes files dragged in from the file manager over the element of `ref` (Kotlin `FileDropZone`): returns whether
 * files hover over it now, and hands the dropped paths to `onDrop`. Without Tauri (mock mode) nothing is dropped.
 */
export function useFileDrop(ref: RefObject<HTMLElement | null>, onDrop: (paths: string[]) => void): boolean {
  const [active, setActive] = useState(false);
  const latest = useRef(onDrop);
  latest.current = onDrop;
  useEffect(() => {
    if (!isTauri()) return;
    const inside = (position: { x: number; y: number }) => {
      const rect = ref.current?.getBoundingClientRect();
      if (!rect) return false;
      const x = position.x / window.devicePixelRatio;
      const y = position.y / window.devicePixelRatio;
      return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
    };
    let stopped = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (payload.type === "leave") return setActive(false);
        const over = inside(payload.position);
        if (payload.type === "drop") {
          setActive(false);
          if (over) latest.current(payload.paths);
          return;
        }
        setActive(over);
      })
      .then((stop) => (stopped ? stop() : (unlisten = stop)));
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [ref]);
  return active;
}
