import ADD from "@material-design-icons/svg/round/add.svg?raw";
import ARCHIVE from "@material-design-icons/svg/round/archive.svg?raw";
import AUTO from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import BOLT from "@material-design-icons/svg/round/bolt.svg?raw";
import CALENDAR from "@material-design-icons/svg/round/calendar_month.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import EDIT from "@material-design-icons/svg/round/edit.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import GRID from "@material-design-icons/svg/round/grid_view.svg?raw";
import INBOX from "@material-design-icons/svg/round/inbox.svg?raw";
import REFRESH from "@material-design-icons/svg/round/refresh.svg?raw";
import SIDEBAR from "@material-design-icons/svg/round/view_sidebar.svg?raw";
import WARNING from "@material-design-icons/svg/round/warning.svg?raw";
import type {LibraryViewDto} from "../../../ipc";
import {ActionMenu, IconButton, ReasonTooltip} from "../../../shared/components";
import {commandKey} from "../../../shared/platform";
import {useStore} from "../../../shared/store";
import {openExport} from "../export/export-state";
import {savedViews, select} from "../state/actions";
import {panels} from "../state/panels";
import type {LibraryState} from "../state/queries";
import {ALL_DOCUMENTS, sameView, type SidebarSection as Section, viewStore} from "../state/view-state";
import {SidebarItem, SidebarSection} from "./sidebar-item";

/**
 * The library's views: the "Library" section and the saved views, no Trash (that is the system trash) and no trees
 * (Kotlin `LibrarySidebar`). The panel button folds the sidebar into the icon rail.
 */
export function LibrarySidebar({ state, folded, width, busyReason }: { state: LibraryState | undefined; folded: boolean; width: number; busyReason: string | null }) {
  const view = useStore(viewStore);
  const label = `${folded ? "Expand" : "Collapse"} sidebar (${commandKey}\\)`;
  const counts = state?.counts;
  const isOpen = (section: Section) => folded || !view.collapsed.includes(section);
  const toggle = (section: Section) =>
    viewStore.set((current) => ({ ...current, collapsed: current.collapsed.includes(section) ? current.collapsed.filter((open) => open !== section) : [...current.collapsed, section] }));
  const views: { view: LibraryViewDto; label: string; icon: string; count: number; warning?: boolean }[] = [
    { view: ALL_DOCUMENTS, label: "All documents", icon: GRID, count: counts?.all ?? 0 },
    { view: { kind: "inbox" }, label: "Inbox", icon: INBOX, count: counts?.inbox ?? 0, warning: true },
    { view: { kind: "needsReview" }, label: "Needs review", icon: AUTO, count: counts?.needsReview ?? 0, warning: true },
    { view: { kind: "recent" }, label: "Added this month", icon: CALENDAR, count: counts?.addedThisMonth ?? 0 },
    { view: { kind: "folders" }, label: "Folders", icon: FOLDER, count: counts?.folders ?? 0 },
  ];
  return (
    <nav className="lib-sidebar" style={{ width }} aria-label="Library views">
      <div className={folded ? "lib-panel-button folded" : "lib-panel-button"}>
        <ReasonTooltip reason={label} beside={folded}>
          <IconButton icon={SIDEBAR} label={label} onClick={() => panels.toggleFolded("sidebar")} />
        </ReasonTooltip>
      </div>
      <div className="lib-sidebar-scroll">
        <SidebarSection title="Library" expanded={isOpen("library")} folded={folded} onToggle={() => toggle("library")} />
        {isOpen("library") &&
          views.map((entry) => (
            <SidebarItem
              key={entry.label}
              label={entry.label}
              icon={entry.icon}
              count={entry.count}
              warning={entry.warning}
              selected={sameView(view.view, entry.view)}
              folded={folded}
              onClick={() => select(entry.view)}
            />
          ))}
        <SidebarSection
          title="Saved views"
          expanded={isOpen("savedViews")}
          folded={folded}
          onToggle={() => toggle("savedViews")}
          add={
            <ReasonTooltip reason={busyReason ?? "Add the standard saved views"}>
              <IconButton icon={ADD} label="Add the standard saved views" small disabled={busyReason !== null} onClick={savedViews.addStandard} />
            </ReasonTooltip>
          }
        />
        {isOpen("savedViews") &&
          (state?.smartFolders ?? []).map((entry) => (
            <SidebarItem
              key={entry.id}
              label={entry.invalid ? `${entry.name} (invalid)` : entry.name}
              icon={entry.invalid ? WARNING : BOLT}
              count={entry.count}
              selected={view.view.kind === "smartFolder" && view.view.id === entry.id}
              folded={folded}
              onClick={() => select({ kind: "smartFolder", id: entry.id })}
              menu={
                <ActionMenu
                  label={`Actions for ${entry.name}`}
                  enabled={busyReason === null}
                  busyReason={busyReason}
                  actions={[
                    { label: "Rename", icon: EDIT, onClick: () => void savedViews.rename(entry.id, entry.name) },
                    { label: "Use the current view", icon: REFRESH, onClick: () => savedViews.replaceWithCurrent(entry.id) },
                    { label: "Export…", icon: ARCHIVE, onClick: () => openExport({ kind: "smartFolder", id: entry.id }, entry.name) },
                    { label: "Delete", icon: DELETE, onClick: () => void savedViews.remove(entry.id, entry.name) },
                  ]}
                />
              }
            />
          ))}
      </div>
    </nav>
  );
}
