import ADD from "@material-design-icons/svg/round/add.svg?raw";
import AUTO from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import MERGE from "@material-design-icons/svg/round/merge_type.svg?raw";
import REFRESH from "@material-design-icons/svg/round/refresh.svg?raw";
import SYNC from "@material-design-icons/svg/round/sync.svg?raw";
import { aiBatches } from "../../features/dashboard/ai-batches";
import { addFilesToOpenFolder, addFilesToTopLevel, libraryTitle } from "../../features/library";
import { operations } from "../../session/operations";
import { busyIn, useSession } from "../../session/session";
import { ActionButtons, waitFor, type ActionButton } from "../../shared/components";
import { useStore } from "../../shared/store";
import { PALETTE_ACTIONS, paletteShortcut, type PaletteActionId } from "../palette/palette-actions";
import { screenLabel, setPaletteOpen, type Screen } from "../screens";
import { SplitButton, WindowToolbar } from "./window-toolbar";

/**
 * The window toolbar of `screen`: its title, the palette field and its actions. The dashboard gets the Scan split
 * button and Add files, the library Add files; the other screens only their title.
 */
export function ScreenToolbar({ screen }: { screen: Screen }) {
  const state = useSession();
  const library = useStore(libraryTitle);
  const busy = busyIn(state, "library");
  const aiBusy = busyIn(state, "ai");
  const item = (action: PaletteActionId, icon: string, waitingFor: string | null, onClick: () => void): ActionButton => ({
    label: PALETTE_ACTIONS[action].label,
    icon,
    disabledReason: waitingFor ? waitFor(waitingFor) : null,
    shortcut: paletteShortcut(action) ?? undefined,
    onClick,
  });
  let actions = null;
  if (screen === "dashboard") {
    const menu = [
      item("syncAll", SYNC, busy, operations.syncToAllDestinations),
      item("mergeAll", MERGE, busy, operations.mergeAllFolders),
      ...(state.aiEnabled ? [item("tagUntagged", AUTO, aiBusy, aiBatches.tagAll)] : []),
    ];
    actions = (
      <>
        <SplitButton main={{ label: "Scan", icon: REFRESH, onClick: operations.scan }} menu={menu} menuLabel="More library actions" busyReason={busy} />
        <AddFilesButton busyReason={busy} onClick={() => void addFilesToTopLevel()} />
      </>
    );
  } else if (screen === "library") {
    actions = <AddFilesButton busyReason={busy} onClick={() => void addFilesToOpenFolder()} />;
  }
  return <WindowToolbar title={screen === "library" ? library : screenLabel(screen)} onOpenPalette={() => setPaletteOpen(true)} actions={actions} />;
}

function AddFilesButton({ busyReason, onClick }: { busyReason: string | null; onClick: () => void }) {
  return <ActionButtons buttons={[{ label: "Add files", icon: ADD, style: "filled", onClick }]} enabled={busyReason === null} busyReason={busyReason} />;
}
