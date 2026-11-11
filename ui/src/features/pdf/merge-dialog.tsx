import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import MERGE_TYPE from "@material-design-icons/svg/round/merge_type.svg?raw";
import { commands, IpcFailure, unwrap, type MergeCandidateDto } from "../../ipc";
import { reportError } from "../../session/session";
import { Button, Dialog, Icon, IconButton, TextField, dialogs } from "../../shared/components";
import { createStore, useStore } from "../../shared/store";

/**
 * "Merge with…": the PDFs to merge in their order (the first is the document the user started from), the search for
 * more, the new file's name and whether the sources go to the trash.
 */
type MergeState = { chosen: MergeCandidateDto[]; query: string; matches: MergeCandidateDto[]; name: string; trashSources: boolean };

const merge = createStore<MergeState | null>(null);

export async function startMerge(documentId: string) {
  try {
    const start = await unwrap(commands.mergeStart(documentId));
    merge.set({ chosen: [start.first], query: "", matches: [], name: start.name, trashSources: false });
    await search("");
  } catch (error) {
    reportError(error);
  }
}

async function search(query: string) {
  const current = merge.get();
  if (!current) return;
  merge.set({ ...current, query });
  const matches = await unwrap(commands.mergeMatches(query, current.chosen.map((candidate) => candidate.documentId))).catch((error: unknown) => {
    reportError(error);
    return [];
  });
  // A later keystroke may have searched again meanwhile.
  merge.set((latest) => (latest && latest.query === query ? { ...latest, matches } : latest));
}

function add(candidate: MergeCandidateDto) {
  merge.set((current) => current && { ...current, chosen: [...current.chosen, candidate] });
  void search(merge.get()?.query ?? "");
}

function remove(documentId: string) {
  merge.set((current) => current && { ...current, chosen: current.chosen.filter((candidate) => candidate.documentId !== documentId) });
}

async function confirmMerge(state: MergeState) {
  const name = state.name.trim();
  if (state.chosen.length < 2 || name === "") return;
  if (state.trashSources) {
    const confirmed = await dialogs.confirm({
      title: `Merge ${state.chosen.length} PDFs`,
      message: "The merged file is created first; then the sources go to the trash in every folder.",
      affected: state.chosen.map((candidate) => candidate.path),
      confirmLabel: "Merge and move to trash",
    });
    if (!confirmed) return;
  }
  try {
    await unwrap(commands.mergeDocuments(state.chosen.map((candidate) => candidate.documentId), name, state.trashSources));
    merge.set(null);
  } catch (error) {
    // Kotlin kept the dialog open without a word while another operation ran.
    if (!(error instanceof IpcFailure && error.code === "busy")) reportError(error);
  }
}

// A stable function: the dialog refocuses its first control whenever `onDismiss` changes.
function closeMerge() {
  merge.set(null);
}

export function MergeDialog() {
  const state = useStore(merge);
  if (!state) return null;
  return (
    <Dialog
      title={
        <span className="column pdf-dialog-heading">
          <Icon svg={MERGE_TYPE} size={24} />
          Merge PDFs
        </span>
      }
      width={600} className="pdf-wide-dialog"
      onDismiss={closeMerge}
      buttons={
        <>
          <Button variant="text" onClick={closeMerge}>Cancel</Button>
          <Button variant="filled" disabled={state.chosen.length < 2 || state.name.trim() === ""} onClick={() => void confirmMerge(state)}>
            Merge {state.chosen.length} PDFs
          </Button>
        </>
      }
    >
      <div className="column gap-s">
        {state.chosen.map((candidate, index) => (
          <div key={candidate.documentId} className="row pdf-merge-row">
            <span className="body-medium ellipsis grow">{`${index + 1}. ${candidate.name}`}</span>
            {index > 0 && <IconButton icon={CLOSE} label={`Leave out ${candidate.name}`} onClick={() => remove(candidate.documentId)} />}
          </div>
        ))}
        <TextField label="Add a PDF: search" value={state.query} labelBackground="var(--surface-high)" onChange={(query) => void search(query)} />
        <div className="pdf-merge-matches scroll">
          {state.matches.map((candidate) => (
            <button key={candidate.documentId} type="button" className="pdf-merge-match body-medium ellipsis" onClick={() => add(candidate)}>
              {candidate.path}
            </button>
          ))}
        </div>
        <TextField label="Name of the merged file" value={state.name} labelBackground="var(--surface-high)" onChange={(name) => merge.set({ ...state, name })} />
        <label className="row">
          <input type="checkbox" className="checkbox" checked={state.trashSources} onChange={(event) => merge.set({ ...state, trashSources: event.target.checked })} />
          <span className="body-medium">Move the merged PDFs to the trash afterwards</span>
        </label>
      </div>
    </Dialog>
  );
}
