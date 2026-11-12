import {useState} from "react";
import {keepPreviousData, useQuery} from "@tanstack/react-query";
import CHAT from "@material-design-icons/svg/round/chat.svg?raw";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import RENAME from "@material-design-icons/svg/round/drive_file_rename_outline.svg?raw";
import FILE_OPEN from "@material-design-icons/svg/round/file_open.svg?raw";
import FOLDER_OPEN from "@material-design-icons/svg/round/folder_open.svg?raw";
import MORE from "@material-design-icons/svg/round/more_horiz.svg?raw";
import MOVE_DOWN from "@material-design-icons/svg/round/move_down.svg?raw";
import NEW_LABEL from "@material-design-icons/svg/round/new_label.svg?raw";
import SUMMARIZE from "@material-design-icons/svg/round/summarize.svg?raw";
import {commands, type QuickLook_Serialize, unwrap} from "../../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, useSession} from "../../../session/session";
import {
    type ActionButton,
    ActionMenu,
    Button,
    dialogs,
    Icon,
    IconButton,
    KindBadge,
    PageThumbnail,
    ReasonTooltip,
    waitFor
} from "../../../shared/components";
import {fileManagerName} from "../../../shared/platform";
import {useStore} from "../../../shared/store";
import {MoreSection} from "../details/more-section";
import {OrganizeBlock} from "./organize/organize-block";
import {
    closeQuickLook,
    editFact,
    endEdit,
    openChat,
    openReview,
    openSummary,
    quickLook,
    quickLookEditing,
    saveFact
} from "../state";
import {InlineTextEditor} from "./inline-text-editor";
import {QuickFactList} from "./quick-facts";
import {SuggestionCard} from "./suggestion-card";

/** Why the parts of the quick look cannot act now: a busy library lane, a busy AI lane. */
export type BusyReasons = { library: string | null; ai: string | null };

/** The quick look panel at the right of the library (380 px); renders nothing while closed or when the document left the catalog. */
export function QuickLookPanel() {
  const { documentId } = useStore(quickLook);
  const look = useQuery({
    queryKey: [LIBRARY_QUERY, "document", "quick-look", documentId],
    queryFn: () => unwrap(commands.quickLook(documentId ?? "")),
    enabled: documentId !== null,
    placeholderData: keepPreviousData,
  });
  const [moreOpen, setMoreOpen] = useState(false);
  if (documentId === null || !look.data) return null;
  return <QuickLookSheet look={look.data} moreOpen={moreOpen} onToggleMore={() => setMoreOpen((open) => !open)} />;
}

function QuickLookSheet({ look, moreOpen, onToggleMore }: { look: QuickLook_Serialize; moreOpen: boolean; onToggleMore: () => void }) {
  const session = useSession();
  const library = busyIn(session, "library");
  const ai = busyIn(session, "ai");
  const busy: BusyReasons = { library: library === null ? null : waitFor(library), ai: ai === null ? null : waitFor(ai) };
  const editing = useStore(quickLookEditing);
  const fact = editing?.documentId === look.documentId ? editing.fact : null;
  return (
    <aside className="quick-look" aria-label="Quick look">
      <QuickLookTopBar look={look} busy={busy} aiEnabled={session.aiEnabled} />
      <div className="divider" />
      <div className="quick-look-body scroll">
        <QuickLookHeader look={look} editingTitle={fact === "title"} busyReason={busy.library} />
        {look.suggestion && (
          <SuggestionCard
            values={look.suggestion}
            busyReason={busy.library}
            onAccept={() => unwrap(commands.acceptSuggestion(look.documentId)).then((problem) => problem && openReview(look.documentId, problem), reportError)}
            onEdit={() => openReview(look.documentId)}
            onDismiss={() => unwrap(commands.dismissSuggestion(look.documentId)).catch(reportError)}
          />
        )}
        <QuickFactList look={look} editing={fact} busyReason={busy.library} />
        <OrganizeBlock documentId={look.documentId} busyReason={busy.library} />
        <MoreSection look={look} open={moreOpen} onToggle={onToggleMore} busyReason={busy.library} />
      </div>
    </aside>
  );
}

/**
 * "Open", then icons to show the file and to ask the AI about it, which never wait, and ⋯ to summarize, tag and
 * classify it, which wait for another AI run, and to rename, move or trash it, which wait for a busy library.
 */
function QuickLookTopBar({ look, busy, aiEnabled }: { look: QuickLook_Serialize; busy: BusyReasons; aiEnabled: boolean }) {
  const id = look.documentId;
  const unreadable = look.readableByAi ? null : "The AI does not read this kind of file";
  const aiActions: ActionButton[] = aiEnabled
    ? [
        { label: "Summarize with AI", icon: SUMMARIZE, disabledReason: unreadable, onClick: () => openSummary(id, look.fileName) },
        { label: "Tag with AI", icon: NEW_LABEL, disabledReason: unreadable ?? busy.ai, onClick: () => run(commands.tagDocumentWithAi(id)) },
        { label: "Classify with AI", icon: AUTO_AWESOME, disabledReason: unreadable ?? busy.ai, onClick: () => run(commands.classifyDocumentWithAi(id)) },
      ]
    : [];
  const actions: ActionButton[] = [
    ...aiActions,
    { label: "Rename…", icon: RENAME, disabledReason: busy.library, onClick: () => void rename(look) },
    { label: "Move…", icon: MOVE_DOWN, disabledReason: busy.library, onClick: () => void move(look) },
    { label: "Move to trash…", icon: DELETE, disabledReason: busy.library, onClick: () => void trash(id) },
  ];
  return (
    <div className="quick-look-top-bar">
      <Button variant="filled" icon={FILE_OPEN} onClick={() => run(commands.openDocumentFile(id))}>Open</Button>
      <OutlinedIconAction icon={FOLDER_OPEN} label={`Show in ${fileManagerName()}`} onClick={() => run(commands.showDocumentInFileManager(id))} />
      {aiEnabled && <OutlinedIconAction icon={CHAT} label="Ask about this document…" disabledReason={unreadable} onClick={() => openChat(id, look.fileName)} />}
      <ActionMenu actions={actions} label={`Rename, move, trash ${look.fileName}…`} icon={MORE} />
      <span className="label-small quick-look-hint nowrap">↑↓ next</span>
      <IconButton icon={CLOSE} label="Close quick look" onClick={closeQuickLook} />
    </div>
  );
}

/** An action as an outlined icon; its tooltip names it, or says why it cannot be used now (Kotlin `ActionIconButton`). */
function OutlinedIconAction({ icon, label, disabledReason = null, onClick }: { icon: string; label: string; disabledReason?: string | null; onClick: () => void }) {
  return (
    <ReasonTooltip reason={disabledReason ?? label}>
      <button type="button" className="quick-look-icon-action" aria-label={label} disabled={disabledReason !== null} onClick={onClick}>
        <Icon svg={icon} />
      </button>
    </ReasonTooltip>
  );
}

/** A large preview of the first page, the title, which a click edits in place, and the path in the library. */
function QuickLookHeader({ look, editingTitle, busyReason }: { look: QuickLook_Serialize; editingTitle: boolean; busyReason: string | null }) {
  const badge = <KindBadge kind={look.kind as "pdf" | "image" | "office" | "other"} size={52} />;
  return (
    <div className="column gap-m">
      <div className="quick-look-preview-row">
        <div className="quick-look-preview">
          {look.primaryFolderId === null ? badge : <PageThumbnail folderId={look.primaryFolderId} path={look.path} fallback={badge} style={{ width: "100%", height: "100%" }} />}
        </div>
      </div>
      <div className="column" style={{ gap: 2 }}>
        {editingTitle ? (
          <InlineTextEditor initial={look.title ?? ""} description="Title" onSave={(typed) => saveFact(look.documentId, "title", typed)} onCancel={endEdit} />
        ) : (
          <ReasonTooltip reason={busyReason}>
            <button type="button" className="quick-look-title title-medium" disabled={busyReason !== null} onClick={() => editFact(look.documentId, "title")}>
              {look.title ?? look.fileName}
            </button>
          </ReasonTooltip>
        )}
        <span className="body-small ellipsis quick-look-path">{look.path}</span>
      </div>
    </div>
  );
}

function run(call: Promise<{ status: "ok"; data: unknown } | { status: "error"; error: { code: string; message: string } }>) {
  unwrap(call).catch(reportError);
}

async function rename(look: QuickLook_Serialize) {
  const name = await dialogs.askText({ title: "Rename file", label: "Name", initialValue: look.fileName, confirmLabel: "Rename" });
  if (name !== null) run(commands.renameDocument(look.documentId, name));
}

async function move(look: QuickLook_Serialize) {
  const folder = await dialogs.askText({
    title: `Move ${look.fileName}`,
    label: "Folder, such as Letters/2024, or / for the top level",
    initialValue: look.folder,
    confirmLabel: "Move",
  });
  if (folder !== null) run(commands.moveDocumentToFolder(look.documentId, folder));
}

async function trash(documentId: string) {
  try {
    const question = await unwrap(commands.trashQuestion(documentId));
    const confirmed = await dialogs.confirm({ title: question.title, message: question.message, affected: question.affected, confirmLabel: question.confirmLabel });
    if (confirmed) await unwrap(commands.moveDocumentToTrash(documentId));
  } catch (error) {
    reportError(error);
  }
}
