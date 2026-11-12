import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import ADD from "@material-design-icons/svg/round/add.svg?raw";
import TASK_ALT from "@material-design-icons/svg/round/task_alt.svg?raw";
import {commands, unwrap} from "../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, useSession} from "../../session/session";
import {
    AppCard,
    Button,
    Menu,
    MenuItem,
    ScreenHeader,
    TextField,
    useMenuAnchor,
    waitFor
} from "../../shared/components";
import {AddDialog, type AddKind} from "./add-dialog";
import {EmergencyList} from "./emergency-list";
import {OverviewCard} from "./overview-card";
import "./emergency.css";

export { AddToEmergencyButton, type AddToEmergencyButtonProps } from "./add-to-emergency-button";

const ADD_KINDS: { kind: AddKind; label: string }[] = [
  { kind: "document", label: "Document…" },
  { kind: "contract", label: "Contract…" },
  { kind: "contact", label: "Contact…" },
  { kind: "note", label: "Note…" },
];

/**
 * The emergency folder: what a partner or relative needs if something happens to the user. Items by group with
 * include switches and "changed" pills, the refresh status, the overview preview, printing and the encrypted ZIP.
 */
export function EmergencyScreen() {
  const session = useSession();
  const query = useQuery({ queryKey: [LIBRARY_QUERY, "emergency"], queryFn: () => unwrap(commands.emergencyState()) });
  const [adding, setAdding] = useState<AddKind | null>(null);
  const [intro, setIntro] = useState<string | null>(null);
  const menu = useMenuAnchor();
  const busy = busyIn(session, "library");
  const busyReason = busy === null ? null : waitFor(busy);
  const state = query.data;
  if (!state) return null;
  const empty = state.groups.every((group) => group.items.length === 0);
  const saveIntro = () => {
    if (intro === null || intro === state.intro) return;
    unwrap(commands.emergencyEdit({ kind: "setIntro", intro })).then(() => setIntro(null), reportError);
  };
  return (
    <div className="screen">
      <div className="emergency-screen">
        <ScreenHeader
          title="Emergency folder"
          subtitle="What a partner or relative needs if something happens to you"
          actions={
            <Button variant="tonal" icon={ADD} disabled={busyReason !== null} title={busyReason ?? undefined} onClick={menu.open}>Add</Button>
          }
        />
        <div className="emergency-columns">
          <div className="column gap-l">
            <AppCard padding={16}>
              <div className="row gap-m">
                <div className="column grow">
                  <span className="title-small">{state.status.title}</span>
                  <span className="body-small muted">{state.status.detail}</span>
                </div>
                {state.status.refreshDue && <span className="emergency-due label-medium">Refresh due</span>}
                <Button icon={TASK_ALT} disabled={empty || busyReason !== null} title={busyReason ?? (empty ? "The folder is empty" : undefined)}
                  onClick={() => void unwrap(commands.emergencyMarkRefreshed()).catch(reportError)}>
                  Mark as refreshed
                </Button>
              </div>
            </AppCard>
            <TextField label="Intro on the overview" value={intro ?? state.intro} multiline rows={2} onChange={setIntro} onBlur={saveIntro}
              support="For example where the paper folder is kept" />
            {empty
              ? <p className="body-medium muted">Add the documents, contracts, contacts and notes a trusted person would need.</p>
              : <EmergencyList groups={state.groups} busyReason={busyReason} />}
          </div>
          <OverviewCard state={state} busyReason={busyReason} />
        </div>
      </div>
      {menu.anchor && (
        <Menu anchor={menu.anchor} align="end" onClose={menu.close}>
          {ADD_KINDS.map(({ kind, label }) => <MenuItem key={kind} label={label} onClick={() => { menu.close(); setAdding(kind); }} />)}
        </Menu>
      )}
      {adding && <AddDialog kind={adding} contracts={state.contracts} onClose={() => setAdding(null)} />}
    </div>
  );
}
