import {useState} from "react";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import LOCK from "@material-design-icons/svg/round/lock.svg?raw";
import PRINT from "@material-design-icons/svg/round/print.svg?raw";
import {commands, type EmergencyStateDto_Serialize, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {Button, Menu, MenuItem, useMenuAnchor} from "../../shared/components";
import {chooseSaveLocation} from "../../shared/platform";
import {ZipDialog} from "./zip-dialog";

/** The overview as it will be printed, with "Print overview", "Save in library" and "Encrypted ZIP…". */
export function OverviewCard({ state, busyReason }: { state: EmergencyStateDto_Serialize; busyReason: string | null }) {
  const [zip, setZip] = useState(false);
  const folders = useMenuAnchor();
  const reason = busyReason ?? (state.included === 0 ? "Switch on at least one item" : null);
  const print = async () => {
    const path = await chooseSaveLocation("Save the overview to print", "Emergency overview.pdf");
    if (path) await unwrap(commands.emergencySaveOverview({ kind: "outside", path }, true)).catch(reportError);
  };
  const saveIn = (folder: string) => {
    folders.close();
    unwrap(commands.emergencySaveOverview({ kind: "library", folder }, false)).catch(reportError);
  };
  const { preview } = state;
  return (
    <div className="column gap-m">
      <div className="emergency-page">
        <h3 className="title-medium">{preview.title}</h3>
        <span className="body-small muted">{preview.subtitle}</span>
        {preview.intro && <p className="body-small">{preview.intro}</p>}
        <table className="body-small">
          <thead><tr><th>Area</th><th>Document / contact</th></tr></thead>
          <tbody>{preview.rows.map((row, index) => <tr key={index}><td>{row.area}</td><td>{row.text}</td></tr>)}</tbody>
        </table>
      </div>
      <div className="row wrap gap-s">
        <Button icon={PRINT} disabled={reason !== null} title={reason ?? undefined} onClick={() => void print()}>Print overview</Button>
        <Button icon={FOLDER} disabled={reason !== null} title={reason ?? undefined} onClick={folders.open}>Save in library</Button>
        <Button variant="filled" icon={LOCK} disabled={reason !== null} title={reason ?? undefined} onClick={() => setZip(true)}>Encrypted ZIP…</Button>
      </div>
      {folders.anchor && (
        <Menu anchor={folders.anchor} onClose={folders.close}>
          {state.folders.map((folder) => <MenuItem key={folder} label={folder === "" ? "Library (top level)" : folder} icon={FOLDER} onClick={() => saveIn(folder)} />)}
        </Menu>
      )}
      {zip && <ZipDialog onClose={() => setZip(false)} />}
    </div>
  );
}
