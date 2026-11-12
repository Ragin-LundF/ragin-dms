import {keepPreviousData, useQuery} from "@tanstack/react-query";
import ADD from "@material-design-icons/svg/round/add.svg?raw";
import ICON from "@material-design-icons/svg/round/forum.svg?raw";
import {commands, unwrap} from "../../ipc";
import {LIBRARY_QUERY} from "../../session/session";
import {Button, dialogs, EmptyState, ScreenHeader, Spinner} from "../../shared/components";
import {useStore} from "../../shared/store";
import {CaseDetail} from "./case-detail";
import {CaseList} from "./case-list";
import {DocumentPickerDialog} from "./document-picker-dialog";
import {LetterDialog} from "../letters";
import {caseLetterStore, casesScreen, startCase} from "./state";
import "./cases.css";

export { ReviewCases } from "./review-cases";
export { draftCaseLetter, openCase, reviewCaseChoice, startCase } from "./state";

/** Cases with follow-ups (feature 4): the Active or Done list on the left, the chosen case on the right. */
export function CasesScreen() {
  const { filter, selected } = useStore(casesScreen);
  const view = useQuery({
    queryKey: [LIBRARY_QUERY, "cases", filter, selected],
    queryFn: () => unwrap(commands.casesView(filter, selected)),
    placeholderData: keepPreviousData,
  });
  const newCase = async () => {
    const title = await dialogs.askText({ title: "New case", label: "Title", initialValue: "", confirmLabel: "Start case" });
    if (title?.trim()) await startCase(title, null, []);
  };
  return (
    <div className="screen">
      <div className="cases-screen">
        <ScreenHeader
          title="Cases"
          subtitle="Letters, replies and reminders of one matter, with status and the next step"
          actions={<Button variant="tonal" icon={ADD} onClick={() => void newCase()}>New case</Button>}
        />
        {!view.data ? (
          <div className="center-fill"><Spinner /></div>
        ) : view.data.activeCount + view.data.doneCount === 0 ? (
          <EmptyState icon={ICON} title="No cases yet" message="Start a case for a matter with letters back and forth, or create one from a document's quick look." style={{ flex: 1 }} />
        ) : (
          <div className="cases-layout">
            <CaseList view={view.data} filter={filter} selected={view.data.detail?.id ?? null} />
            {view.data.detail ? <CaseDetail detail={view.data.detail} /> : <EmptyState icon={ICON} title="Nothing here" message="No case in this list." />}
          </div>
        )}
      </div>
    </div>
  );
}

/** The dialogs cases open from anywhere (reminder letters, the document picker); mounted once by the shell. */
export function CaseDialogs() {
  return (
    <>
      <CaseLetterHost />
      <DocumentPickerDialog />
    </>
  );
}

/** The reminder letter of a case, saved into that case as sent. */
function CaseLetterHost() {
  const letter = useStore(caseLetterStore);
  if (!letter) return null;
  return (
    <LetterDialog
      kind="reminder"
      documentIds={letter.documentIds}
      recipient={letter.recipient ?? undefined}
      caseId={letter.caseId ?? undefined}
      onClose={() => caseLetterStore.set(null)}
    />
  );
}
