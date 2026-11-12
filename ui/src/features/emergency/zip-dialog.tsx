import {useState} from "react";
import LOCK from "@material-design-icons/svg/round/lock.svg?raw";
import {commands, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {Button, Dialog, TextField} from "../../shared/components";
import {chooseSaveLocation} from "../../shared/platform";

const MIN_LENGTH = 8;

/** Asks for the password twice, then where to save the AES-256 encrypted ZIP; the password is never kept. */
export function ZipDialog({ onClose }: { onClose: () => void }) {
  const [password, setPassword] = useState("");
  const [repeated, setRepeated] = useState("");
  const [writing, setWriting] = useState(false);
  const problem = password.length < MIN_LENGTH
    ? `At least ${MIN_LENGTH} characters`
    : repeated !== password ? "The passwords differ" : null;
  const write = async () => {
    const path = await chooseSaveLocation("Save the encrypted emergency folder", "Emergency folder.zip");
    if (!path) return;
    setWriting(true);
    try {
      if (await unwrap(commands.emergencyExportZip(path, password))) onClose();
    } catch (error) {
      reportError(error);
    } finally {
      setWriting(false);
    }
  };
  return (
    <Dialog title="Create encrypted folder" width={460} onDismiss={onClose} buttons={
      <>
        <Button variant="text" onClick={onClose}>Cancel</Button>
        <Button variant="filled" icon={LOCK} disabled={problem !== null || writing} onClick={() => void write()}>Create ZIP</Button>
      </>
    }>
      <div className="column gap-m">
        <p className="body-medium">The included documents and the overview go into a ZIP encrypted with AES-256.</p>
        <TextField label="Password for the trusted person" type="password" value={password} onChange={setPassword}
          error={password.length > 0 && password.length < MIN_LENGTH ? problem : null} autoFocus labelBackground="var(--surface-high)" />
        <TextField label="Repeat the password" type="password" value={repeated} onChange={setRepeated}
          error={repeated.length > 0 && repeated !== password ? "The passwords differ" : null} labelBackground="var(--surface-high)" />
        <p className="body-small muted">Give the password on paper, separately from the ZIP. It is not stored anywhere. The password protects the content of the files, not their names: anyone with the ZIP can see the file names. Open the ZIP with 7-Zip, Keka or The Unarchiver.</p>
      </div>
    </Dialog>
  );
}
