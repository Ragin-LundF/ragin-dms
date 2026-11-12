import CONTENT_CUT from "@material-design-icons/svg/round/content_cut.svg?raw";
import {useState} from "react";
import {commands, pageUrl, unwrap} from "../../ipc";
import {reportError} from "../../session/session";
import {Button, Dialog, dialogs, Icon, Spinner} from "../../shared/components";
import {useStore} from "../../shared/store";
import {partsOf, splitDialog, type SplitDialogState, type SplitPart, toggledCut} from "./split-state";

const THUMBNAIL_WIDTH = 120;
const SWATCHES = ["var(--chart-1)", "var(--chart-2)", "var(--chart-3)", "var(--chart-4)", "var(--tertiary)", "var(--chart-neutral)"];

/**
 * "Split a stack" (prototype "Split a scanned stack"): the pages as a strip with the AI's cuts, a click on a gap
 * adds or removes a cut, and below it the documents the split makes. Nothing changes until "Split and review".
 */
export function SplitDialog() {
  const state = useStore(splitDialog);
  if (!state) return null;
  const proposal = state.proposal;
  const parts = proposal ? partsOf(proposal.pageCount, state.cuts, proposal.parts) : [];
  return (
    <Dialog
      title={`Split ${state.proposal?.name ?? (state.name || "a stack")}`}
      width={960}
      className="split-dialog"
      onDismiss={closeSplit}
      buttons={
        <>
          <Button variant="text" onClick={closeSplit}>Cancel</Button>
          <Button variant="filled" icon={CONTENT_CUT} disabled={parts.length < 2} title={parts.length < 2 ? "Mark at least one cut to split the stack" : undefined} onClick={() => void apply(state, parts)}>
            Split and review
          </Button>
        </>
      }
    >
      {state.problem && <p className="body-medium split-problem">{state.problem}</p>}
      {!state.problem && !proposal && (
        <div className="row gap-s split-waiting">
          <Spinner small />
          <span className="body-medium muted">The AI is looking for the documents in the stack…</span>
        </div>
      )}
      {proposal && (
        <div className="column gap-m">
          {proposal.note && <p className="body-small split-note">{proposal.note}</p>}
          <span className="body-small muted">✂ marks a cut the AI proposes. Click a gap between two pages to add or remove a cut.</span>
          <PageStrip documentId={state.documentId} pageCount={proposal.pageCount} cuts={state.cuts} onToggle={(page) => splitDialog.set({ ...state, cuts: toggledCut(state.cuts, page) })} />
          <span className="title-small">Documents after the split <span className="muted body-small">{parts.length} documents</span></span>
          <div className="column gap-s">
            {parts.map((part, index) => <PartRow key={part.firstPage} part={part} index={index} />)}
          </div>
          {parts.length > 1 && (
            <label className="row">
              <input type="checkbox" className="checkbox" checked={state.trashOriginal} onChange={(event) => splitDialog.set({ ...state, trashOriginal: event.target.checked })} />
              <span className="body-medium">Move the original to the trash after splitting</span>
            </label>
          )}
        </div>
      )}
    </Dialog>
  );
}

// A stable function: the dialog refocuses its first control whenever `onDismiss` changes.
function closeSplit() {
  splitDialog.set(null);
}

async function apply(state: SplitDialogState, parts: SplitPart[]) {
  if (!state.proposal) return;
  if (state.trashOriginal) {
    const confirmed = await dialogs.confirm({
      title: `Split ${state.proposal.name} into ${parts.length} documents`,
      message: `The new documents are created first; then ${state.proposal.name} goes to the trash in every folder.`,
      affected: [state.proposal.name],
      confirmLabel: "Split and move to trash",
    });
    if (!confirmed) return;
  }
  try {
    await unwrap(commands.splitApply({ documentId: state.documentId, parts, model: state.proposal.model, trashOriginal: state.trashOriginal }));
    splitDialog.set(null);
  } catch (error) {
    reportError(error);
  }
}

function PageStrip({ documentId, pageCount, cuts, onToggle }: { documentId: string; pageCount: number; cuts: number[]; onToggle: (page: number) => void }) {
  return (
    <div className="row split-strip scroll">
      {Array.from({ length: pageCount }, (_, page) => (
        <div key={page} className="row split-step">
          {page > 0 && (
            <button
              type="button"
              className={cuts.includes(page) ? "split-cut on" : "split-cut"}
              aria-pressed={cuts.includes(page)}
              aria-label={cuts.includes(page) ? `Remove the cut before page ${page + 1}` : `Cut before page ${page + 1}`}
              title="Cut here"
              onClick={() => onToggle(page)}
            >
              {cuts.includes(page) && <Icon svg={CONTENT_CUT} />}
            </button>
          )}
          <PagePicture documentId={documentId} page={page} />
        </div>
      ))}
    </div>
  );
}

function PagePicture({ documentId, page }: { documentId: string; page: number }) {
  const [loaded, setLoaded] = useState<"loading" | "loaded" | "failed">("loading");
  return (
    <div className="column split-page">
      <div className="split-page-picture">
        {loaded !== "loaded" && <span className="title-medium muted">{page + 1}</span>}
        {loaded !== "failed" && (
          <img
            src={pageUrl(documentId, page, THUMBNAIL_WIDTH)}
            alt={`Page ${page + 1}`}
            draggable={false}
            style={{ display: loaded === "loaded" ? undefined : "none" }}
            onLoad={() => setLoaded("loaded")}
            onError={() => setLoaded("failed")}
          />
        )}
      </div>
      <span className="label-small muted">{page + 1}</span>
    </div>
  );
}

function PartRow({ part, index }: { part: SplitPart; index: number }) {
  const pages = part.lastPage > part.firstPage ? `Pages ${part.firstPage + 1}–${part.lastPage + 1}` : `Page ${part.firstPage + 1}`;
  const facts = [pages, part.categoryLabel, part.correspondent, part.documentDate].filter(Boolean).join(" · ");
  return (
    <div className="row gap-m split-part">
      <span className="split-swatch" style={{ background: SWATCHES[index % SWATCHES.length] }} />
      <div className="column grow">
        <span className="title-small ellipsis">{part.title ?? `New document (${pages.toLowerCase()})`}</span>
        <span className="body-small muted ellipsis">{facts}</span>
      </div>
      <span className={part.confidence === null ? "split-pill label-small" : "split-pill ai label-small"}>
        {part.confidence === null ? "AI suggests after the split" : `AI ${part.confidence} % sure`}
      </span>
    </div>
  );
}
