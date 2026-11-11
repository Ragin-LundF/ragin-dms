import {keepPreviousData, useQuery} from "@tanstack/react-query";
import ADD_LINK from "@material-design-icons/svg/round/add_link.svg?raw";
import {commands, unwrap} from "../../../ipc";
import {LIBRARY_QUERY, reportError} from "../../../session/session";
import {Button, Dialog, TextField} from "../../../shared/components";
import {useStore} from "../../../shared/store";
import {DialogHeading} from "../review/dialog-heading";
import {closeLinkPick as close, linkPickStore} from "../state";

const DIALOG_SURFACE = "var(--surface-high)";

/** "Link to…": search a document by name, title or correspondent, give the link a label, click it (Kotlin `LinkPickerDialog`). */
export function LinkPickerDialog() {
  const pick = useStore(linkPickStore);
  const matches = useQuery({
    queryKey: [LIBRARY_QUERY, "document", "link-candidates", pick?.documentId, pick?.query],
    queryFn: () => unwrap(commands.linkCandidatesOf(pick?.documentId ?? "", pick?.query ?? "")),
    enabled: pick !== null,
    placeholderData: keepPreviousData,
  });
  if (!pick) return null;
  const link = (otherId: string) => {
    close();
    unwrap(commands.linkDocuments(pick.documentId, otherId, pick.label)).catch(reportError);
  };
  return (
    <Dialog title={<DialogHeading icon={ADD_LINK} title="Link to another document" />} width={560} className="document-dialog" onDismiss={close} buttons={<Button variant="text" onClick={close}>Cancel</Button>}>
      <div className="column gap-m">
        <TextField label="Search" value={pick.query} onChange={(query) => linkPickStore.set({ ...pick, query })} autoFocus labelBackground={DIALOG_SURFACE} />
        <TextField label={'Label, such as "reminder" (optional)'} value={pick.label} onChange={(label) => linkPickStore.set({ ...pick, label })} labelBackground={DIALOG_SURFACE} />
        <div className="scroll link-matches">
          {(matches.data ?? []).map((candidate) => (
            <button key={candidate.documentId} type="button" className="link-match" onClick={() => link(candidate.documentId)}>
              <span className="body-large ellipsis">{candidate.name}</span>
              <span className="body-small muted ellipsis">{candidate.path}</span>
            </button>
          ))}
        </div>
      </div>
    </Dialog>
  );
}
