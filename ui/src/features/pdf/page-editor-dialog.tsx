import CHEVRON_LEFT from "@material-design-icons/svg/round/chevron_left.svg?raw";
import CHEVRON_RIGHT from "@material-design-icons/svg/round/chevron_right.svg?raw";
import CONTENT_CUT from "@material-design-icons/svg/round/content_cut.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import RESTORE_FROM_TRASH from "@material-design-icons/svg/round/restore_from_trash.svg?raw";
import ROTATE_LEFT from "@material-design-icons/svg/round/rotate_left.svg?raw";
import ROTATE_RIGHT from "@material-design-icons/svg/round/rotate_right.svg?raw";
import { useState } from "react";
import { commands, IpcFailure, pageUrl, unwrap, type PageItemDto } from "../../ipc";
import { reportError } from "../../session/session";
import { Button, Dialog, IconButton, Spinner, dialogs } from "../../shared/components";
import { useStore } from "../../shared/store";
import { changePage, editSummary, moved, pageEditor, turned, type PageEditorState } from "./page-editor";

const QUARTER = 90;
const THUMBNAIL_WIDTH = 220;

/**
 * The pages of a PDF as a strip: turn, remove and move each page, and cut after a page to split the document there.
 * Nothing changes until the user applies.
 */
export function PageEditorDialog() {
  const state = useStore(pageEditor);
  if (!state) return null;
  const pages = state.pages ?? [];
  const { kept, edited, splits } = editSummary(pages);
  const setPages = (next: PageItemDto[]) => pageEditor.set({ ...state, pages: next });
  return (
    <Dialog
      title={`Pages of ${state.name}`}
      width={960} className="pdf-wide-dialog"
      onDismiss={closePageEditor}
      buttons={
        <>
          <Button variant="text" onClick={closePageEditor}>Cancel</Button>
          <Button variant="filled" disabled={kept === 0 || !(edited || splits > 0)} onClick={() => void apply(state, splits)}>
            {splits > 0 ? `Apply and split into ${splits + 1}` : "Apply"}
          </Button>
        </>
      }
    >
      {state.pages === null ? (
        <Spinner />
      ) : (
        <div className="column gap-m">
          <div className="row gap-m pdf-page-strip scroll">
            {pages.map((page, position) => (
              <PageCard key={page.source} documentId={state.documentId} page={page} position={position} pages={pages} onChange={setPages} />
            ))}
          </div>
          {splits > 0 && (
            <label className="row">
              <input type="checkbox" className="checkbox" checked={state.trashOriginal} onChange={(event) => pageEditor.set({ ...state, trashOriginal: event.target.checked })} />
              <span className="body-medium">Move the original to the trash after splitting</span>
            </label>
          )}
        </div>
      )}
    </Dialog>
  );
}

// A stable function: the dialog refocuses its first control whenever `onDismiss` changes.
function closePageEditor() {
  pageEditor.set(null);
}

async function apply(state: PageEditorState, splits: number) {
  if (splits > 0 && state.trashOriginal) {
    const confirmed = await dialogs.confirm({
      title: `Split ${state.name} into ${splits + 1} documents`,
      message: `The new documents are created first; then ${state.name} goes to the trash in every folder.`,
      affected: [state.name],
      confirmLabel: "Split and move to trash",
    });
    if (!confirmed) return;
  }
  try {
    await unwrap(commands.editPdfPages(state.documentId, state.pages ?? [], state.trashOriginal));
    pageEditor.set(null);
  } catch (error) {
    // Kotlin kept the editor open without a word while another operation ran.
    if (!(error instanceof IpcFailure && error.code === "busy")) reportError(error);
  }
}

function PageCard({ documentId, page, position, pages, onChange }: {
  documentId: string;
  page: PageItemDto;
  position: number;
  pages: PageItemDto[];
  onChange: (pages: PageItemDto[]) => void;
}) {
  const change = (update: (page: PageItemDto) => PageItemDto) => onChange(changePage(pages, position, update));
  return (
    <div className="column pdf-page-card" style={{ opacity: page.deleted ? 0.35 : 1 }}>
      <PagePicture documentId={documentId} page={page} />
      <span className="label-medium">Page {page.source + 1}</span>
      <div className="row">
        <IconButton icon={CHEVRON_LEFT} label="Move left" onClick={() => onChange(moved(pages, position, -1))} />
        <IconButton icon={ROTATE_LEFT} label="Turn left" onClick={() => change((current) => turned(current, -QUARTER))} />
        <IconButton icon={ROTATE_RIGHT} label="Turn right" onClick={() => change((current) => turned(current, QUARTER))} />
        <IconButton icon={CHEVRON_RIGHT} label="Move right" onClick={() => onChange(moved(pages, position, 1))} />
      </div>
      <div className="row">
        <IconButton
          icon={page.deleted ? RESTORE_FROM_TRASH : DELETE}
          label={page.deleted ? "Keep page" : "Remove page"}
          onClick={() => change((current) => ({ ...current, deleted: !current.deleted }))}
        />
        <IconButton
          icon={CONTENT_CUT}
          label="Split after this page"
          className={page.splitAfter ? "pdf-split-on" : undefined}
          onClick={() => change((current) => ({ ...current, splitAfter: !current.splitAfter }))}
        />
      </div>
      {page.splitAfter && <span className="label-small primary-text">Split here</span>}
    </div>
  );
}

/** The page's picture, turned as edited; its number until the picture is there or when there is none. */
function PagePicture({ documentId, page }: { documentId: string; page: PageItemDto }) {
  const [loaded, setLoaded] = useState<"loading" | "loaded" | "failed">("loading");
  return (
    <div className="pdf-page-picture">
      {loaded !== "loaded" && <span className="headline-medium">{page.source + 1}</span>}
      {loaded !== "failed" && (
        <img
          src={pageUrl(documentId, page.source, THUMBNAIL_WIDTH)}
          alt={`Page ${page.source + 1}`}
          draggable={false}
          style={{ display: loaded === "loaded" ? undefined : "none", transform: `rotate(${page.rotation}deg)` }}
          onLoad={() => setLoaded("loaded")}
          onError={() => setLoaded("failed")}
        />
      )}
    </div>
  );
}
