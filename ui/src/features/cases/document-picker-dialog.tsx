import {keepPreviousData, useQuery} from "@tanstack/react-query";
import ADD from "@material-design-icons/svg/round/note_add.svg?raw";
import {type CaseDirectionDto, commands, unwrap} from "../../ipc";
import {LIBRARY_QUERY} from "../../session/session";
import {Button, Dialog, Icon, TextField} from "../../shared/components";
import {useStore} from "../../shared/store";
import {change, documentPickStore} from "./state";

/** "Add document": search a document by name, title or correspondent and add it as received or sent. */
export function DocumentPickerDialog() {
  const pick = useStore(documentPickStore);
  const matches = useQuery({
    queryKey: [LIBRARY_QUERY, "cases", "candidates", pick?.caseId, pick?.query],
    queryFn: () => unwrap(commands.casesDocumentCandidates(pick?.caseId ?? "", pick?.query ?? "")),
    enabled: pick !== null,
    placeholderData: keepPreviousData,
  });
  if (!pick) return null;
  const close = () => documentPickStore.set(null);
  const add = (documentId: string, direction: CaseDirectionDto) => {
    close();
    void change(unwrap(commands.casesAddDocument(pick.caseId, documentId, direction)));
  };
  return (
    <Dialog
      title={<span className="row gap-s"><Icon svg={ADD} /> Add a document to the case</span>}
      width={560}
      onDismiss={close}
      buttons={<Button variant="text" onClick={close}>Cancel</Button>}
    >
      <div className="column gap-m">
        <TextField label="Search" value={pick.query} onChange={(query) => documentPickStore.set({ ...pick, query })} autoFocus labelBackground="var(--surface-high)" />
        <div className="scroll case-candidates">
          {(matches.data ?? []).map((candidate) => (
            <div key={candidate.documentId} className="row gap-s case-candidate">
              <span className="column grow">
                <span className="body-large ellipsis">{candidate.name}</span>
                <span className="body-small muted ellipsis">{candidate.path}</span>
              </span>
              <Button small onClick={() => add(candidate.documentId, "received")}>Received</Button>
              <Button small onClick={() => add(candidate.documentId, "sent")}>Sent</Button>
            </div>
          ))}
        </div>
      </div>
    </Dialog>
  );
}
