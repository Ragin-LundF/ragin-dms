import SUMMARIZE from "@material-design-icons/svg/round/summarize.svg?raw";
import { Button, Dialog, Spinner } from "../../../shared/components";
import { useStore } from "../../../shared/store";
import { DialogHeading } from "../review/dialog-heading";
import { closeSummary, openChat, summaryStore } from "../state";

/** The AI's summary of a document while it is written; the text can be selected (Kotlin `SummaryDialog`). */
export function SummaryDialog() {
  const summary = useStore(summaryStore);
  if (!summary) return null;
  const askQuestion = () => {
    closeSummary();
    openChat(summary.documentId, summary.fileName);
  };
  return (
    <Dialog
      title={<DialogHeading icon={SUMMARIZE} title={`Summary of ${summary.fileName}`} />}
      width={620} className="document-dialog"
      onDismiss={closeSummary}
      buttons={
        <>
          <Button variant="text" onClick={closeSummary}>Close</Button>
          <Button variant="text" onClick={askQuestion}>Ask a question…</Button>
        </>
      }
    >
      <div className="column gap-m scroll summary-body">
        {summary.working && (
          <div className="row gap-m">
            <Spinner small />
            <span className="body-medium">The AI is reading the document…</span>
          </div>
        )}
        {summary.text !== null && (
          <>
            <span className="body-large selectable pre-wrap">{summary.text}</span>
            <span className="body-small muted">{`Written by ${summary.model}. Check important facts in the document.`}</span>
          </>
        )}
        {summary.problem && <span className="body-medium error-text">{summary.problem}</span>}
      </div>
    </Dialog>
  );
}
