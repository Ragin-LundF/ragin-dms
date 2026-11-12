import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import ICON from "@material-design-icons/svg/round/receipt_long.svg?raw";
import {commands, type TaxItemDto_Serialize} from "../../ipc";
import {busyIn, LIBRARY_QUERY, useSession} from "../../session/session";
import {Button, Chip, EmptyState, ScreenHeader, waitFor} from "../../shared/components";
import {ExportDialog} from "../library";
import {LetterDialog} from "../letters";
import {TaxAreaGroup} from "./area-group";
import {TaxDocumentPicker} from "./document-picker";
import {TaxSummary} from "./summary-cards";
import {taxActions} from "./tax-actions";
import "./tax.css";

export { TaxAssignDialog } from "./assign-dialog";

/** The tax year folder: what the return of a year needs, what is there, what is missing, and the sums per area. */
export function TaxScreen() {
  const session = useSession();
  const [year, setYear] = useState<number | null>(null);
  const [havingIt, setHavingIt] = useState<TaxItemDto_Serialize | null>(null);
  const [requesting, setRequesting] = useState<TaxItemDto_Serialize | null>(null);
  const query = useQuery({ queryKey: [LIBRARY_QUERY, "tax", year], queryFn: () => commands.taxYearView(year) });
  const view = query.data?.status === "ok" ? query.data.data : null;
  const busy = busyIn(session, "library");
  if (!view) return null;
  const disabledReason = busy !== null ? waitFor(busy) : view.opened ? null : `Start the ${view.year} checklist first`;
  return (
    <div className="screen">
      <div className="tax-screen">
        <ScreenHeader
          title={`Tax year ${view.year}`}
          subtitle="What the tax return needs, what is there, and what is still missing"
          actions={
            <div className="row gap-xs" role="radiogroup" aria-label="Tax year">
              {view.years.map((candidate) => (
                <Chip key={candidate} label={String(candidate)} selected={candidate === view.year} onClick={() => setYear(candidate)} />
              ))}
            </div>
          }
        />
        {!view.opened && (
          <div className="tax-preview row gap-s">
            <span className="body-medium">This is a preview of the {view.year} checklist. Nothing is saved until you start it.</span>
            <Button variant="filled" small disabled={busy !== null} onClick={() => taxActions.open(view.year)}>
              Start the {view.year} checklist
            </Button>
          </div>
        )}
        {view.areas.length === 0 ? (
          <EmptyState icon={ICON} title="Nothing on the list yet" message="Assign documents to this tax year, or add items of your own." style={{ flex: 1 }} />
        ) : (
          <div className="tax-layout">
            <div className="tax-areas">
              {view.areas.map((group) => (
                <TaxAreaGroup key={group.area.area} year={view.year} group={group} disabledReason={disabledReason} onHaveIt={setHavingIt} onRequest={setRequesting} />
              ))}
            </div>
            <TaxSummary view={view} />
          </div>
        )}
      </div>
      {havingIt && (
        <TaxDocumentPicker
          year={view.year}
          itemId={havingIt.id}
          itemLabel={havingIt.label}
          onDismiss={() => setHavingIt(null)}
          onPick={(documentId) => {
            taxActions.change(view.year, { kind: "linkDocument", itemId: havingIt.id, documentId });
            setHavingIt(null);
          }}
        />
      )}
      {requesting && (
        <LetterDialog
          kind="request"
          documentIds={[]}
          recipient={requesting.correspondent ?? undefined}
          subjectFacts={[`${requesting.label} for the tax return ${view.year}`]}
          onClose={() => setRequesting(null)}
        />
      )}
      <ExportDialog />
    </div>
  );
}
