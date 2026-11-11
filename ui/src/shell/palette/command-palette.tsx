import {useEffect, useRef, useState} from "react";
import {keepPreviousData, useQuery} from "@tanstack/react-query";
import SEARCH from "@material-design-icons/svg/round/search.svg?raw";
import {commands, unwrap} from "../../ipc";
import {askLibrary} from "../../features/ask/navigation";
import {aiBatches} from "../../features/dashboard/ai-batches";
import {quickLook} from "../../features/document";
import {addFilesToTopLevel} from "../../features/library";
import {openDocument, openLibrary} from "../../features/library/navigation";
import {startRebuild} from "../../features/pdf";
import {operations} from "../../session/operations";
import {busyIn, LIBRARY_QUERY, report, useSession} from "../../session/session";
import {Icon} from "../../shared/components";
import {commandHeld, commandKey} from "../../shared/platform";
import {goTo, setPaletteOpen, useNavigation} from "../screens";
import type {PaletteActionId} from "./palette-actions";
import {moveSelection, paletteGroups, type PaletteItem} from "./palette-items";

/** The command palette over the window (⌘K). ↑/↓ move, ↵ chooses, ⌘↵ asks the library, Esc closes. */
export function CommandPalette() {
  const { paletteOpen } = useNavigation();
  return paletteOpen ? <PaletteDialog /> : null;
}

function PaletteDialog() {
  const state = useSession();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const list = useRef<HTMLDivElement>(null);
  const matches = useQuery({
    queryKey: [LIBRARY_QUERY, "palette", query.trim()],
    queryFn: () => unwrap(commands.paletteMatches(query)),
    enabled: query.trim().length > 0,
    placeholderData: keepPreviousData,
  });
  const busy = { library: busyIn(state, "library") ?? undefined, ai: busyIn(state, "ai") ?? undefined };
  const groups = paletteGroups(query, query.trim() ? matches.data ?? { documents: [], places: [] } : { documents: [], places: [] }, busy, state.aiEnabled, commandKey);
  const items = groups.flatMap((group) => group.items);
  const index = items.length === 0 ? 0 : Math.min(selected, items.length - 1);
  useEffect(() => {
    list.current?.querySelector(`[data-index="${index}"]`)?.scrollIntoView({ block: "nearest" });
  }, [index]);
  const close = () => setPaletteOpen(false);
  const choose = (item: PaletteItem | undefined) => {
    // An action that waits for its lane is never run; the palette stays open so its reason stays visible.
    if (!item || (item.kind === "run" && item.disabledReason)) return;
    close();
    run(item);
  };
  const onKey = (event: React.KeyboardEvent) => {
    if (event.key === "ArrowDown") setSelected(moveSelection(index, 1, items.length));
    else if (event.key === "ArrowUp") setSelected(moveSelection(index, -1, items.length));
    else if (event.key === "Escape") close();
    else if (event.key === "Enter" && commandHeld(event)) {
      if (!state.aiEnabled || !query.trim()) return;
      close();
      askLibrary(query.trim());
    } else if (event.key === "Enter") choose(items[index]);
    else return;
    event.preventDefault();
    event.stopPropagation();
  };
  let position = -1;
  return (
    <div className="scrim" onMouseDown={(event) => event.target === event.currentTarget && close()}>
      <div className="palette" role="dialog" aria-label="Command palette">
        <div className="field">
          <span className="field-icon"><Icon svg={SEARCH} /></span>
          <input
            autoFocus
            value={query}
            placeholder="Search documents, tags, actions…"
            aria-label="Search documents, tags, actions…"
            onChange={(event) => {
              setQuery(event.target.value);
              setSelected(0);
            }}
            onKeyDown={onKey}
          />
        </div>
        {items.length === 0 ? (
          <div className="body-medium muted" style={{ padding: "var(--space-l)" }}>Nothing found</div>
        ) : (
          <div ref={list} className="palette-list scroll">
            {groups.map((group, groupIndex) => (
              <div key={group.title} className="column" style={{ gap: 1 }}>
                {groupIndex > 0 && <div className="divider" style={{ margin: "var(--space-xs) 0" }} />}
                <span className="label-medium muted" style={{ padding: "var(--space-xs) var(--space-m)" }}>{group.title}</span>
                {group.items.map((item) => {
                  position++;
                  const rowIndex = position;
                  const waiting = item.kind === "run" && item.disabledReason !== null;
                  return (
                    <button key={`${item.kind}-${item.label}`} type="button" data-index={rowIndex} className={rowIndex === index ? "palette-row selected" : "palette-row"} disabled={waiting} onMouseMove={() => setSelected(rowIndex)} onClick={() => choose(item)}>
                      <span className="column grow">
                        <span className="body-medium ellipsis palette-label">{item.label}</span>
                        {item.detail && <span className="body-small muted ellipsis">{item.detail}</span>}
                      </span>
                      {item.hint && <span className="label-medium muted">{item.hint}</span>}
                    </button>
                  );
                })}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

/** Runs what a line stands for, through the same functions as the buttons (Kotlin `PaletteCommands`). */
function run(item: PaletteItem) {
  switch (item.kind) {
    case "openDocument":
      return openDocument(item.documentId);
    case "goTo":
      return openLibrary({ kind: "show", selection: item.selection, query: "" });
    case "ask":
      return askLibrary(item.question);
    case "run":
      return runAction(item.action);
  }
}

function runAction(action: PaletteActionId) {
  const actions: Record<PaletteActionId, () => void> = {
    scan: operations.scan,
    syncAll: operations.syncToAllDestinations,
    mergeAll: operations.mergeAllFolders,
    retryFailed: operations.retryFailedOperations,
    detectDates: operations.detectDocumentDates,
    addFiles: () => void addFilesToTopLevel(),
    // The AI batches show their progress and results on the dashboard, so it comes to the front.
    tagUntagged: () => {
      goTo("dashboard");
      aiBatches.tagAll();
    },
    classifyUncategorized: () => {
      goTo("dashboard");
      aiBatches.classifyAll();
    },
    convertToRealPdf: () => {
      const shown = quickLook.get().documentId;
      if (shown) startRebuild(shown);
      else report({ text: "Open a scanned PDF in the library first", isError: true });
    },
    openDashboard: () => goTo("dashboard"),
    openTasks: () => goTo("tasks"),
    openLibrary: () => goTo("library"),
    openSettings: () => goTo("settings"),
  };
  actions[action]();
}
