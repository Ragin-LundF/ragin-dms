import {useCallback, useEffect, useRef} from "react";
import type {LibraryKindDto} from "../../../ipc";
import {createStore, useStore} from "../../../shared/store";

/** What the user drags inside the library (Kotlin `DragItem`): a document, or a folder of the folder view. */
export type DragItem =
  | { kind: "document"; id: string; title: string; documentKind: LibraryKindDto }
  | { kind: "folder"; path: string };

/**
 * Where a dragged item can be dropped (Kotlin `DropTarget`). Only folders take drops; `place` tells apart the same
 * folder shown twice at once, such as a tree row and a breadcrumb segment.
 */
export type DropTarget = { kind: "folder"; path: string; place: "tree" | "tile" | "breadcrumb" };

type DragState = { item: DragItem | null; x: number; y: number; hovered: DropTarget | null };

/** A press becomes a drag after the pointer moved this far, so a click still clicks. */
const DRAG_SLOP = 4;

export const dragStore = createStore<DragState>({ item: null, x: 0, y: 0, hovered: null });

const targets = new Map<string, { target: DropTarget; element: HTMLElement }>();
let onDrop: (item: DragItem, target: DropTarget) => void = () => undefined;

function keyOf(target: DropTarget): string {
  return `${target.place}:${target.path}`;
}

/** Whether `target` takes `item`: a folder never goes into itself (into a subfolder is taken, so the drop says why not). */
export function takes(item: DragItem, target: DropTarget): boolean {
  return item.kind === "document" || target.path !== item.path;
}

function targetAt(item: DragItem, x: number, y: number): DropTarget | null {
  for (const { target, element } of targets.values()) {
    const rect = element.getBoundingClientRect();
    if (x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom && takes(item, target)) return target;
  }
  return null;
}

/** Sets what a drop does; the library screen owns it while it is shown. */
export function setDropHandler(handler: (item: DragItem, target: DropTarget) => void) {
  onDrop = handler;
}

/**
 * Starts following a press on a draggable element; once it moved a few pixels it is a drag of `item`, and the click
 * that would end it is swallowed. Releasing over a target that takes the item drops it there.
 */
export function beginPress(event: React.PointerEvent, item: DragItem) {
  if (event.button !== 0) return;
  const startX = event.clientX;
  const startY = event.clientY;
  let dragging = false;
  const move = (moved: PointerEvent) => {
    if (!dragging && Math.hypot(moved.clientX - startX, moved.clientY - startY) < DRAG_SLOP) return;
    dragging = true;
    dragStore.set({ item, x: moved.clientX, y: moved.clientY, hovered: targetAt(item, moved.clientX, moved.clientY) });
  };
  const up = () => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", up);
    window.removeEventListener("pointercancel", cancel);
    if (!dragging) return;
    const swallow = (click: MouseEvent) => {
      click.stopPropagation();
      click.preventDefault();
    };
    window.addEventListener("click", swallow, { capture: true, once: true });
    window.setTimeout(() => window.removeEventListener("click", swallow, { capture: true }), 0);
    const { hovered } = dragStore.get();
    dragStore.set({ item: null, x: 0, y: 0, hovered: null });
    if (hovered) onDrop(item, hovered);
  };
  const cancel = () => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", up);
    window.removeEventListener("pointercancel", cancel);
    dragStore.set({ item: null, x: 0, y: 0, hovered: null });
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", up);
  window.addEventListener("pointercancel", cancel);
}

/** Registers the element of `ref` as drop `target`; returns whether a dragged item hovers over it now. */
export function useDropTarget(target: DropTarget): { ref: (element: HTMLElement | null) => void; hovered: boolean } {
  const key = keyOf(target);
  const latest = useRef(target);
  latest.current = target;
  const ref = useCallback(
    (element: HTMLElement | null) => {
      if (element) targets.set(key, { target: latest.current, element });
      else targets.delete(key);
    },
    [key],
  );
  useEffect(() => () => void targets.delete(key), [key]);
  const drag = useStore(dragStore);
  return { ref, hovered: drag.item !== null && drag.hovered !== null && keyOf(drag.hovered) === key };
}

/** What dropping `item` on `target` does, in words, for the drag preview (Kotlin `DropTexts`). */
export function dropAction(item: DragItem, target: DropTarget): string {
  const where = target.path || "the top level";
  return item.kind === "folder" ? `Move into ${where}` : `Move to ${where}`;
}
