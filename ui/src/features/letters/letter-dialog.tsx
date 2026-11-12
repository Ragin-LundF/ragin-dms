import {useEffect, useState} from "react";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import DESCRIPTION from "@material-design-icons/svg/round/description.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import {commands, describeError, type LetterDraftDto, type LetterKindDto, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {Button, Dialog, Icon, Menu, MenuItem, Spinner, TextField, useMenuAnchor} from "../../shared/components";

export type LetterDialogProps = {
  kind: LetterKindDto;
  documentIds: string[];
  recipient?: string;
  subjectFacts?: string[];
  onClose: () => void;
  onSaved?: (documentId: string) => void;
  /** The case the letter belongs to: it is saved into it as sent. Without one, the user may start a case. */
  caseId?: string;
};

/** After saving: start a case (when the letter has none) and follow it up after some days. */
type CaseOptions = { startCase: boolean; followUp: boolean; days: string };

const TITLES: Record<LetterKindDto, string> = {
  cancellation: "Cancellation letter",
  objection: "Objection",
  reminder: "Reminder",
  request: "Request",
  reply: "Reply",
};

/** What the dialog edits: the draft's texts, the recipient's address as typed and the chosen folder. */
type Edit = { recipient: string; address: string; subject: string; body: string; folder: string };

/**
 * A letter drafted from documents (by the AI, or from a template when AI is off or fails), edited here and saved as
 * a PDF into a library folder, catalogued and linked to the documents.
 */
export function LetterDialog({ kind, documentIds, recipient, subjectFacts, onClose, onSaved, caseId }: LetterDialogProps) {
  const [draft, setDraft] = useState<LetterDraftDto | null>(null);
  const [options, setOptions] = useState<CaseOptions>({ startCase: false, followUp: caseId !== undefined, days: "14" });
  const [edit, setEdit] = useState<Edit | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let current = true;
    unwrap(commands.letterDraft({ kind, documentIds, recipient: recipient ?? null, subjectFacts: subjectFacts ?? [] }))
      .then((drafted) => {
        if (!current) return;
        setDraft(drafted);
        setEdit({
          recipient: drafted.recipient.name,
          address: drafted.recipient.addressLines.join("\n"),
          subject: drafted.subject,
          body: drafted.body,
          folder: drafted.folder,
        });
      })
      .catch((error: unknown) => current && setProblem(describeError(error)));
    return () => {
      current = false;
    };
    // One draft per opened dialog; the inputs are fixed while it is open.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function save(letter: LetterDraftDto, values: Edit) {
    setSaving(true);
    try {
      const saved = await unwrap(commands.letterSave({
        documentIds,
        folder: values.folder,
        recipient: { name: values.recipient, addressLines: values.address.split("\n") },
        subject: values.subject,
        body: values.body,
        language: letter.language,
      }));
      await fileInCase(saved.documentId);
      onSaved?.(saved.documentId);
      onClose();
    } catch (error) {
      reportError(error);
      setSaving(false);
    }
  }

  async function fileInCase(letterId: string) {
    if (caseId === undefined && !options.startCase) return;
    const days = Number.parseInt(options.days, 10);
    await unwrap(commands.casesFileLetter({
      letterId,
      sourceIds: documentIds,
      caseId: caseId ?? null,
      startCase: options.startCase,
      followUpDays: options.followUp && days > 0 ? days : null,
    })).catch(reportError);
  }

  const canSave = draft !== null && edit !== null && edit.subject.trim() !== "" && !saving;
  return (
    <Dialog
      title={`${TITLES[kind]} (draft)`}
      width={640}
      className="letter-dialog"
      onDismiss={onClose}
      buttons={
        <>
          <Button variant="text" onClick={onClose}>Cancel</Button>
          <Button variant="filled" icon={DESCRIPTION} disabled={!canSave} onClick={() => draft && edit && void save(draft, edit)}>
            Save as PDF
          </Button>
        </>
      }
    >
      {problem && <p className="body-medium letter-problem">{problem}</p>}
      {!problem && (!draft || !edit) && (
        <div className="row gap-s letter-waiting">
          <Spinner small />
          <span className="body-medium muted">Drafting the letter…</span>
        </div>
      )}
      {draft && edit && <LetterForm draft={draft} edit={edit} onChange={setEdit} />}
      {draft && edit && <CaseChoice inCase={caseId !== undefined} options={options} onChange={setOptions} />}
    </Dialog>
  );
}

function LetterForm({ draft, edit, onChange }: { draft: LetterDraftDto; edit: Edit; onChange: (edit: Edit) => void }) {
  const set = (change: Partial<Edit>) => onChange({ ...edit, ...change });
  return (
    <div className="column letter-form">
      {draft.sender && (
        <div className="body-small muted ellipsis">{[draft.sender.name, ...draft.sender.addressLines].join(" · ")}</div>
      )}
      <TextField label="Recipient" value={edit.recipient} labelBackground="var(--surface-high)" onChange={(recipient) => set({ recipient })} />
      <TextField label="Address" value={edit.address} multiline rows={2} labelBackground="var(--surface-high)" onChange={(address) => set({ address })} />
      <TextField label="Subject" value={edit.subject} labelBackground="var(--surface-high)" error={edit.subject.trim() === "" ? "The letter needs a subject" : null} onChange={(subject) => set({ subject })} />
      <TextField label="Letter" value={edit.body} multiline rows={12} labelBackground="var(--surface-high)" onChange={(body) => set({ body })} />
      <div className="row gap-s">
        <span className={draft.draftedByAi ? "letter-origin ai label-medium" : "letter-origin label-medium"}>
          {draft.draftedByAi && <Icon svg={AUTO_AWESOME} />}
          {draft.origin}
        </span>
        <span className="grow" />
        <FolderChoice folders={draft.folders} folder={edit.folder} onChange={(folder) => set({ folder })} />
      </div>
      <p className="body-small muted letter-note">
        {draft.draftedByAi ? "Written by your AI from the documents. " : ""}
        Check and edit it; saving adds it to the library as a PDF, linked to its documents.
      </p>
    </div>
  );
}

function FolderChoice({ folders, folder, onChange }: { folders: string[]; folder: string; onChange: (folder: string) => void }) {
  const menu = useMenuAnchor();
  const label = (path: string) => (path === "" ? "Library (top level)" : path);
  return (
    <>
      <Button variant="text" icon={FOLDER} small title="Folder to save the letter in" onClick={menu.open}>
        <span className="ellipsis letter-folder">{label(folder)}</span>
      </Button>
      {menu.anchor && (
        <Menu anchor={menu.anchor} align="end" onClose={menu.close}>
          {folders.map((path) => (
            <MenuItem key={path} label={label(path)} icon={FOLDER} onClick={() => { onChange(path); menu.close(); }} />
          ))}
        </Menu>
      )}
    </>
  );
}

/** "Start a case" for a letter outside a case, and "Add follow-up in N days" once it is in one. */
function CaseChoice({ inCase, options, onChange }: { inCase: boolean; options: CaseOptions; onChange: (options: CaseOptions) => void }) {
  const withCase = inCase || options.startCase;
  return (
    <div className="row gap-m letter-case-options body-medium">
      {inCase ? (
        <span className="muted">Saved into the case as sent.</span>
      ) : (
        <label className="row gap-s">
          <input type="checkbox" className="checkbox" checked={options.startCase} onChange={(event) => onChange({ ...options, startCase: event.target.checked })} />
          Start a case
        </label>
      )}
      {withCase && (
        <label className="row gap-s">
          <input type="checkbox" className="checkbox" checked={options.followUp} onChange={(event) => onChange({ ...options, followUp: event.target.checked })} />
          Add follow-up in
          <input type="number" min={1} max={365} className="letter-days" aria-label="Days until the follow-up" value={options.days}
            disabled={!options.followUp} onChange={(event) => onChange({ ...options, days: event.target.value })} />
          days
        </label>
      )}
    </div>
  );
}
