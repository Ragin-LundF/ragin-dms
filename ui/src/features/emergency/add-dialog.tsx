import {useState} from "react";
import {keepPreviousData, useQuery} from "@tanstack/react-query";
import {commands, type EmergencyChoiceDto, type EmergencyEditDto, type EmergencyGroupDto, unwrap} from "../../ipc";
import {LIBRARY_QUERY, reportError} from "../../session/session";
import {Button, Chip, Dialog, TextField} from "../../shared/components";
import {GROUPS} from "./groups";

export type AddKind = "document" | "contract" | "contact" | "note";
type Contact = { name: string; role: string; phone: string; email: string; note: string };
const SURFACE = "var(--surface-high)";
const TITLES: Record<AddKind, string> = { document: "Add document", contract: "Add contract", contact: "Add contact", note: "Add note" };

/** Adds a document (searched by name), a contract, a contact or a note to a group of the emergency folder. */
export function AddDialog({ kind, contracts, onClose }: { kind: AddKind; contracts: EmergencyChoiceDto[]; onClose: () => void }) {
  const [group, setGroup] = useState<EmergencyGroupDto>(kind === "contact" ? "contacts" : kind === "contract" ? "contracts" : "personal");
  const [query, setQuery] = useState("");
  const [contact, setContact] = useState<Contact>({ name: "", role: "", phone: "", email: "", note: "" });
  const [note, setNote] = useState("");
  const candidates = useQuery({
    queryKey: [LIBRARY_QUERY, "emergency", "candidates", query],
    queryFn: () => unwrap(commands.emergencyDocumentCandidates(query)),
    enabled: kind === "document",
    placeholderData: keepPreviousData,
  });
  const add = (edit: EmergencyEditDto) => {
    unwrap(commands.emergencyEdit(edit)).then((done) => done && onClose(), reportError);
  };
  const set = (change: Partial<Contact>) => setContact({ ...contact, ...change });
  const typed = kind === "contact" ? contact.name.trim() !== "" : kind === "note" ? note.trim() !== "" : false;
  return (
    <Dialog title={TITLES[kind]} width={540} onDismiss={onClose} className="emergency-add" buttons={
      <>
        <Button variant="text" onClick={onClose}>Cancel</Button>
        {(kind === "contact" || kind === "note") && (
          <Button variant="filled" disabled={!typed}
            onClick={() => add(kind === "contact" ? { kind: "addContact", group, ...contact } : { kind: "addNote", group, text: note })}>
            Add
          </Button>
        )}
      </>
    }>
      <div className="column gap-m">
        <div className="row gap-s emergency-groups">
          {GROUPS.map((choice) => <Chip key={choice.group} label={choice.label} selected={group === choice.group} onClick={() => setGroup(choice.group)} />)}
        </div>
        {kind === "document" && (
          <>
            <TextField label="Search" value={query} onChange={setQuery} autoFocus labelBackground={SURFACE} />
            <Choices choices={candidates.data ?? []} empty="No matching document" onPick={(id) => add({ kind: "addDocument", group, documentId: id })} />
          </>
        )}
        {kind === "contract" && (
          <Choices choices={contracts} empty="Every contract is in the folder already" onPick={(id) => add({ kind: "addContract", group, contractId: id })} />
        )}
        {kind === "contact" && (
          <>
            <TextField label="Name" value={contact.name} onChange={(name) => set({ name })} autoFocus labelBackground={SURFACE} />
            <TextField label="Role, such as tax adviser" value={contact.role} onChange={(role) => set({ role })} labelBackground={SURFACE} />
            <TextField label="Phone" value={contact.phone} onChange={(phone) => set({ phone })} labelBackground={SURFACE} />
            <TextField label="Email" value={contact.email} onChange={(email) => set({ email })} labelBackground={SURFACE} />
            <TextField label="Note" value={contact.note} onChange={(value) => set({ note: value })} labelBackground={SURFACE} />
          </>
        )}
        {kind === "note" && (
          <TextField label="Note, such as where the originals are" value={note} onChange={setNote} multiline rows={4} autoFocus labelBackground={SURFACE} />
        )}
      </div>
    </Dialog>
  );
}

function Choices({ choices, empty, onPick }: { choices: EmergencyChoiceDto[]; empty: string; onPick: (id: string) => void }) {
  if (choices.length === 0) return <p className="body-medium muted">{empty}</p>;
  return (
    <div className="scroll emergency-choices">
      {choices.map((choice) => (
        <button key={choice.id} type="button" className="emergency-choice" onClick={() => onPick(choice.id)}>
          <span className="body-large ellipsis">{choice.name}</span>
          <span className="body-small muted ellipsis">{choice.detail}</span>
        </button>
      ))}
    </div>
  );
}
