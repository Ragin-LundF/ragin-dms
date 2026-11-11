import {useEffect, useRef} from "react";
import {closeQuickLook, openQuickLook, quickLook, QuickLookPanel} from "../document";
import {busyIn, useSession} from "../../session/session";
import {ResizeHandle, waitFor} from "../../shared/components";
import {useStore} from "../../shared/store";
import {takeRequest, useNavigation} from "../../shell/screens";
import {DocumentArea} from "./documents/document-area";
import type {DocumentCallbacks, DocumentClick} from "./documents/document-grid";
import {DragPreview} from "./documents/drag-preview";
import {ExportDialog} from "./export/export-dialog";
import {FilterBar} from "./filter/filter-bar";
import {diskFoldersStore, folderActions} from "./folders/folder-state";
import {FolderPickerDialog} from "./folders/folder-picker";
import {FolderView} from "./folders/folder-view";
import {setDropHandler} from "./interaction/drag";
import {focusSearch, handleLibraryKey, toggleQuickLook} from "./interaction/keys";
import {libraryCommands, libraryTitle} from "./library-stores";
import type {LibraryRequest} from "./navigation";
import {LibrarySidebar} from "./sidebar/library-sidebar";
import {bulk, chosenIds} from "./state/actions";
import {panelOf, panels, panelStore} from "./state/panels";
import {shownStore, useDiskFolders, useLibraryState} from "./state/queries";
import {chosen, selection, selectionStore} from "./state/selection";
import {ALL_DOCUMENTS, opening, viewStore, withFilter} from "./state/view-state";

/**
 * The library (Kotlin `LibraryScreen`): the sidebar of views (foldable to an icon rail, resizable), and on the right
 * the search with the facet chips above the documents, or the folder view while "Folders" is selected. Quick look
 * opens over the right pane at its right edge. Keys outside the text fields follow `LibraryKeys`.
 */
export function LibraryScreen() {
  const view = useStore(viewStore);
  const session = useSession();
  const busy = busyIn(session, "library");
  const aiBusy = busyIn(session, "ai");
  const busyReason = busy === null ? null : waitFor(busy);
  const state = useLibraryState(view).data;
  const foldersOnDisk = useDiskFolders();
  const ticks = useStore(selectionStore);
  const inspected = useStore(quickLook).documentId;
  const sidebar = panelOf(useStore(panelStore), "sidebar");
  const root = useRef<HTMLDivElement>(null);
  const main = useRef<HTMLDivElement>(null);
  const request = useNavigation().request;
  const order = state?.order ?? [];
  const shownTicks = new Set(chosen(ticks, order));
  const primaryFolderId = session.folders.find((folder) => folder.primary)?.id ?? null;

  useEffect(() => diskFoldersStore.set(foldersOnDisk), [foldersOnDisk]);
  useEffect(() => libraryTitle.set(state?.title ?? "All documents"), [state?.title]);
  useMountedLibrary(root);
  useEffect(() => {
    if (request?.screen !== "library") return;
    const taken = takeRequest<LibraryRequest>("library");
    if (taken) apply(taken);
  }, [request]);
  // Quick look walks the documents in the order they are shown, also after a filter changed.
  const orderKey = order.join("\n");
  useEffect(() => {
    const shown = quickLook.get().documentId;
    if (shown !== null && state) openQuickLook(shown, state.order);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [orderKey]);

  const callbacks: DocumentCallbacks = {
    selected: shownTicks,
    inspected,
    folderId: primaryFolderId,
    onClick: (id: string, click: DocumentClick) => {
      if (click === "toggle") selection.toggle(id);
      else if (click === "range") selection.extend(id, order);
      else if (inspected === id) closeQuickLook();
      else openQuickLook(id, order);
    },
    onToggle: selection.toggle,
  };

  return (
    <div
      ref={root}
      className="library"
      tabIndex={-1}
      onMouseDown={(event) => {
        if (isTextInput(event.target)) return;
        // Buttons do not take the focus, so the library's keys keep working; a press in the right pane takes it back.
        event.preventDefault();
        if (main.current?.contains(event.target as Node)) root.current?.focus();
      }}
    >
      <LibrarySidebar state={state} folded={sidebar.folded} width={sidebar.folded ? 60 : sidebar.width} busyReason={busyReason} />
      {sidebar.folded ? (
        <div className="vdivider" />
      ) : (
        <ResizeHandle onResize={(delta) => panels.resize("sidebar", delta)} onResizeEnd={() => panels.endResize("sidebar")} onDoubleClick={() => panels.resetWidth("sidebar")} />
      )}
      <div ref={main} className="lib-main">
        {view.view.kind === "folders" ? (
          <FolderView foldersOnDisk={foldersOnDisk} callbacks={callbacks} busyReason={busyReason} />
        ) : (
          <div className="lib-documents">
            <FilterBar view={view} state={state} busyReason={busyReason} />
            {state && (
              <DocumentArea
                view={view}
                state={state}
                callbacks={callbacks}
                ticked={shownTicks.size}
                busyReason={busyReason}
                aiBusyReason={aiBusy === null ? null : waitFor(aiBusy)}
                aiEnabled={session.aiEnabled}
              />
            )}
          </div>
        )}
        <div className="lib-quick-look">
          <QuickLookPanel />
        </div>
      </div>
      <DragPreview selected={shownTicks} />
      <FolderPickerDialog foldersOnDisk={foldersOnDisk} />
      <ExportDialog />
    </div>
  );
}

/** Connects the menu, the keys and the drops while the screen is shown, and takes the keyboard focus. */
function useMountedLibrary(root: React.RefObject<HTMLDivElement | null>) {
  useEffect(() => {
    panels.load();
    root.current?.focus();
    libraryCommands.set({
      toggleSidebar: () => panels.toggleFolded("sidebar"),
      focusSearch,
      toggleQuickLook,
      selectAll: () => selection.selectAll(shownOrder()),
      clearSelection: selection.clear,
      newFolder: () => void folderActions.newFolder(),
      renameFolder: folderActions.renameOpenFolder,
      deleteFolder: folderActions.deleteOpenFolder,
    });
    setDropHandler((item, target) => {
      if (item.kind === "folder") return folderActions.moveInto(item.path, target.path);
      // A ticked document takes every other ticked one along.
      const ticked = chosenIds();
      bulk.moveTo(target.path, ticked.includes(item.id) ? ticked : [item.id]);
    });
    const onKey = (event: KeyboardEvent) => {
      const active = document.activeElement;
      const inLibrary = active === document.body || (root.current?.contains(active) ?? false);
      if (!inLibrary || event.defaultPrevented) return;
      if (handleLibraryKey(event)) event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      libraryCommands.set(null);
    };
  }, [root]);
}

function shownOrder(): string[] {
  return shownStore.get()?.order ?? [];
}

/** Shows what the dashboard or the palette asked for (Kotlin `LibraryViewModel.show` / `reveal`). */
function apply(request: LibraryRequest) {
  selection.clear();
  if (request.kind === "reveal") {
    viewStore.set((state) => opening(state, ALL_DOCUMENTS));
    openQuickLook(request.documentId, shownOrder());
    return;
  }
  viewStore.set((state) => {
    const opened = opening(state, request.selection);
    const year = request.year == null ? null : { kind: "year" as const, year: request.year, andEarlier: request.yearAndEarlier ?? false };
    return { ...opened, query: request.query, filters: year ? withFilter(opened.filters, year) : opened.filters };
  });
  closeQuickLook();
}

function isTextInput(target: EventTarget): boolean {
  const element = target as HTMLElement;
  return element.tagName === "INPUT" || element.tagName === "TEXTAREA" || element.isContentEditable;
}
