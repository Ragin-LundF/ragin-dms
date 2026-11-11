import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import MOVE_TO_INBOX from "@material-design-icons/svg/round/move_to_inbox.svg?raw";
import SETTINGS from "@material-design-icons/svg/round/settings.svg?raw";
import {commands, type InboxStateDto_Serialize, unwrap} from "../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, useSession} from "../../session/session";
import {ActionButtons, EmptyState, ScreenHeader, waitFor} from "../../shared/components";
import {goTo} from "../../shell/screens";
import {openReview} from "../document";
import {type FolderPick, FolderPickerDialog} from "./folder-picker-dialog";
import {InboxRow} from "./inbox-row";
import "./inbox.css";

/**
 * The inbox: every document dropped into the inbox folder of any library folder, to analyze with the AI or to move
 * by hand. While it is off, it leads to the switch in Settings.
 */
export function InboxScreen() {
  const session = useSession();
  const inbox = useQuery({ queryKey: [LIBRARY_QUERY, "inbox", session.aiEnabled], queryFn: () => commands.inboxState() });
  const [pick, setPick] = useState<FolderPick | null>(null);
  const busy = busyIn(session, "library");
  const aiBusy = busyIn(session, "ai");
  const busyReason = busy === null ? null : waitFor(busy);
  const aiBusyReason = aiBusy === null ? null : waitFor(aiBusy);
  const state = inbox.data;
  if (!state) return null;
  const startMove = (documentId: string) => {
    const row = state.rows.find((candidate) => candidate.documentId === documentId);
    if (row) setPick({ documentId, documentName: row.name, folder: row.prefill ?? state.recentTargets[0] ?? "", fileName: row.name });
  };
  const confirmMove = () => {
    if (!pick) return;
    unwrap(commands.fileInboxDocument(pick.documentId, pick.folder, pick.fileName)).then((started) => started && setPick(null), reportError);
  };
  return (
    <div className="inbox-screen">
      <ScreenHeader
        title="Inbox"
        subtitle={state.enabled ? `${state.rows.length} documents in ${state.path} of your library folders` : "Switched off"}
        actions={
          state.enabled && state.aiAnalysis && (
            <ActionButtons
              buttons={[
                {
                  label: "Analyze all",
                  icon: AUTO_AWESOME,
                  style: "filled",
                  disabledReason: aiBusyReason ?? (state.rows.length === 0 ? "The inbox is empty" : null),
                  onClick: () => void unwrap(commands.analyzeInbox()).catch(reportError),
                },
              ]}
            />
          )
        }
      />
      <InboxBody state={state} busyReason={busyReason} aiBusyReason={aiBusyReason} onMove={startMove} />
      {pick && (
        <FolderPickerDialog
          pick={pick}
          folders={state.folders}
          recent={state.recentTargets}
          onChange={(folder, fileName) => setPick({ ...pick, folder, fileName })}
          onConfirm={confirmMove}
          onDismiss={() => setPick(null)}
        />
      )}
    </div>
  );
}

function InboxBody({ state, busyReason, aiBusyReason, onMove }: { state: InboxStateDto_Serialize; busyReason: string | null; aiBusyReason: string | null; onMove: (documentId: string) => void }) {
  if (!state.enabled) {
    return (
      <EmptyState
        icon={MOVE_TO_INBOX}
        title="The inbox is off"
        message="Switch it on in Settings: a folder in your library, such as Inbox, where you drop new documents on any device. Each one is then analyzed by the AI or filed by hand."
        action={<ActionButtons buttons={[{ label: "Open settings", icon: SETTINGS, onClick: () => goTo("settings") }]} />}
      />
    );
  }
  if (state.rows.length === 0) {
    return (
      <EmptyState
        icon={MOVE_TO_INBOX}
        title="Nothing in the inbox"
        message={`Drop files into ${state.path} in any library folder. They show up here after the next scan.`}
      />
    );
  }
  return (
    <div className="inbox-rows scroll">
      {state.rows.map((row) => (
        <InboxRow
          key={row.documentId}
          row={row}
          aiAnalysis={state.aiAnalysis}
          busyReason={busyReason}
          aiBusyReason={aiBusyReason}
          onAnalyze={() => void unwrap(commands.analyzeInboxDocument(row.documentId)).catch(reportError)}
          onReview={() => openReview(row.documentId)}
          onMove={() => onMove(row.documentId)}
        />
      ))}
    </div>
  );
}
