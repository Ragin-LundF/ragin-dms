import {useEffect, useState} from "react";
import {getCurrentWebview} from "@tauri-apps/api/webview";

/**
 * Files dragged from the file manager onto the window while the screen is shown (Kotlin `FileDropZone`): `true`
 * while they hover, `onDrop` with their paths once dropped.
 */
// shortcut: area-local until the shell offers a shared drop hook.
export function useFileDrop(onDrop: (paths: string[]) => void): boolean {
  const [hovering, setHovering] = useState(false);
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (payload.type === "enter" || payload.type === "over") setHovering(true);
        if (payload.type === "leave") setHovering(false);
        if (payload.type === "drop") {
          setHovering(false);
          if (payload.paths.length > 0) onDrop(payload.paths);
        }
      })
      .then((stop) => (disposed ? stop() : (unlisten = stop)))
      .catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [onDrop]);
  return hovering;
}
