import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import OPEN_IN_NEW from "@material-design-icons/svg/round/open_in_new.svg?raw";
import { commands, unwrap, type ReviewRow_Serialize } from "../../../ipc";
import { reportError } from "../../../session/session";
import { Button, Dialog, TextField } from "../../../shared/components";
import { useStore } from "../../../shared/store";
import { closeReview as close, reviewStore, type OpenReview } from "../state";
import { DialogHeading } from "./dialog-heading";
import { categoryHint } from "./review-hints";
import { SuggestingTextField } from "./suggesting-text-field";

/**
 * The review of an AI suggestion (Kotlin `ReviewDialog` + `ReviewFields`): every suggested value, editable, approved
 * row by row. Nothing changes before "Apply checked values"; "Later" keeps the suggestion.
 */
export function ReviewDialog() {
  const open = useStore(reviewStore);
  if (!open) return null;
  const { review, problem } = open;
  const id = review.documentId;
  const approve = () =>
    unwrap(commands.approveReview(id, open.rows)).then((refused) => (refused === null ? close() : reviewStore.set({ ...open, problem: refused })), reportError);
  const reject = () => unwrap(commands.rejectReview(id)).then((started) => started && close(), reportError);
  return (
    <Dialog
      title={<DialogHeading icon={AUTO_AWESOME} title={`AI suggestion for ${review.documentName}`} />}
      width={680} className="document-dialog"
      onDismiss={close}
      buttons={
        <>
          <Button variant="text" icon={OPEN_IN_NEW} onClick={() => unwrap(commands.openDocumentFile(id)).catch(reportError)}>Open document</Button>
          <Button variant="text" onClick={reject}>Reject suggestion</Button>
          <Button variant="text" onClick={close}>Later</Button>
          <Button variant="filled" onClick={approve}>Apply checked values</Button>
        </>
      }
    >
      <div className="review-fields scroll">
        <span className="body-small muted">{review.source}</span>
        {review.rows.map((row, index) => (
          <ReviewFieldRow key={row.key} row={row} open={open} index={index} />
        ))}
        {problem && <span className="body-medium error-text">{problem}</span>}
      </div>
    </Dialog>
  );
}

/** One suggested value: take it or not, edit it (which checks it), and see what the document has now. */
function ReviewFieldRow({ row, open, index }: { row: ReviewRow_Serialize; open: OpenReview; index: number }) {
  const draft = open.rows[index] ?? { key: row.key, accepted: false, value: "" };
  const update = (change: Partial<typeof draft>) =>
    reviewStore.set({ ...open, problem: null, rows: open.rows.map((current, position) => (position === index ? { ...current, ...change } : current)) });
  const change = (value: string) => update({ value, accepted: true });
  const hint = row.key === "category" ? categoryHint(draft.value, row.options, row.hint) : row.hint;
  const now = `Now: ${row.current === "" ? "—" : row.current}`;
  return (
    <div className="review-row">
      <input type="checkbox" className="checkbox" aria-label={row.label} checked={draft.accepted} onChange={(event) => update({ accepted: event.target.checked })} />
      <div className="column grow" style={{ minWidth: 0 }}>
        {row.options.length === 0 ? (
          <TextField label={row.label} value={draft.value} onChange={change} labelBackground="var(--surface-high)" />
        ) : (
          <SuggestingTextField label={row.label} value={draft.value} options={row.options} onChange={change} />
        )}
        <span className="body-small muted nowrap ellipsis">{hint ? `${now} · ${hint}` : now}</span>
      </div>
    </div>
  );
}
