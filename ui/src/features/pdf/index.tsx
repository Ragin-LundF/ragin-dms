import { MergeDialog, startMerge } from "./merge-dialog";
import { PageEditorDialog } from "./page-editor-dialog";
import { RebuildCompareDialog } from "./rebuild-compare-dialog";
import "./pdf.css";

export { PdfSection, startRebuild } from "./pdf-section";

/** Starts merging `documentId` with others (Kotlin `PdfActions.startMerge`). */
export function openMerge(documentId: string) {
  void startMerge(documentId);
}

/** Every PDF dialog (page editor, merge, rebuild compare); mounted once by the shell. */
export function PdfDialogs() {
  return (
    <>
      <PageEditorDialog />
      <MergeDialog />
      <RebuildCompareDialog />
    </>
  );
}
