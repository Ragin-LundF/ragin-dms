import CONTENT_CUT from "@material-design-icons/svg/round/content_cut.svg?raw";
import AUTO_FIX_HIGH from "@material-design-icons/svg/round/auto_fix_high.svg?raw";
import DOCUMENT_SCANNER from "@material-design-icons/svg/round/document_scanner.svg?raw";
import LAYERS from "@material-design-icons/svg/round/layers.svg?raw";
import MERGE_TYPE from "@material-design-icons/svg/round/merge_type.svg?raw";
import TEXT_SNIPPET from "@material-design-icons/svg/round/text_snippet.svg?raw";
import {useQuery} from "@tanstack/react-query";
import {commands, unwrap} from "../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, useSession} from "../../session/session";
import {type ActionButton, ActionButtons, SectionHeader, waitFor} from "../../shared/components";
import {openSplit} from "../split";
import {startMerge} from "./merge-dialog";
import {openPageEditor} from "./page-editor";

/**
 * The PDF tools of quick look: edit the pages, merge with other PDFs, make a scan searchable, and convert a scan
 * without text to a real PDF, which needs the AI. Nothing for other kinds of documents.
 */
export function PdfSection({ documentId }: { documentId: string }) {
  const session = useSession();
  const tools = useQuery({ queryKey: [LIBRARY_QUERY, "pdf-tools", documentId], queryFn: () => unwrap(commands.pdfTools(documentId)) });
  const data = tools.data;
  if (!data?.pdf) return null;
  const busy = busyIn(session, "library");
  const aiBusy = busyIn(session, "ai");
  const searchable = (precise: boolean) => () => void unwrap(commands.makeSearchable(documentId, precise)).catch(reportError);
  const buttons: ActionButton[] = [
    { label: "Edit pages…", icon: LAYERS, onClick: () => void openPageEditor(documentId, data.name) },
    { label: "Merge with…", icon: MERGE_TYPE, onClick: () => void startMerge(documentId) },
    { label: "Make searchable", icon: TEXT_SNIPPET, onClick: searchable(false) },
    { label: "Precise OCR", icon: DOCUMENT_SCANNER, disabledReason: data.ocrAvailable ? null : "Install ocrmypdf to use it", onClick: searchable(true) },
  ];
  if (session.aiEnabled) {
    buttons.push({ label: "Split stack…", icon: CONTENT_CUT, disabledReason: aiBusy === null ? null : waitFor(aiBusy), onClick: () => void openSplit(documentId, data.name) });
  }
  if (session.aiEnabled && data.rebuildable) {
    buttons.push({ label: "Convert to real PDF", icon: AUTO_FIX_HIGH, disabledReason: aiBusy === null ? null : waitFor(aiBusy), onClick: () => startRebuild(documentId) });
  }
  return (
    <>
      <SectionHeader title="PDF" />
      <ActionButtons buttons={buttons} enabled={busy === null} busyReason={busy === null ? null : waitFor(busy)} />
    </>
  );
}

/** Starts "Convert to real PDF" for `documentId`; the comparison opens when it is done. */
export function startRebuild(documentId: string) {
  void unwrap(commands.startRebuild(documentId)).catch(reportError);
}
