import {useState} from "react";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import EDIT from "@material-design-icons/svg/round/edit.svg?raw";
import EDIT_NOTE from "@material-design-icons/svg/round/edit_note.svg?raw";
import FOLDER_OPEN from "@material-design-icons/svg/round/folder_open.svg?raw";
import {commands, type ContractCardDto_Serialize, unwrap} from "../../ipc";
import {busyIn, reportError, useSession} from "../../session/session";
import {AppCard, Button, dialogs, waitFor} from "../../shared/components";
import {AddToEmergencyButton} from "../emergency";
import {LetterDialog} from "../letters";
import {LogoLetter, Pills} from "./contract-card";
import {ContractEditor} from "./contract-editor";
import {showDocumentsOf} from "./contracts";
import {chartColour, PriceChart} from "./price-chart";
import {TermsReview} from "./terms-review";

type Open = "letter" | "editor" | "terms" | null;

function localToday(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
}

function Figure({ label, value }: { label: string; value: string | null }) {
  return (
    <div className="column">
      <span className="body-small muted">{label}</span>
      <span className="title-small">{value ?? "–"}</span>
    </div>
  );
}

/** The selected contract: its prices, notice, price history and what can be done with it. */
export function ContractDetail({ card, open, onOpen }: { card: ContractCardDto_Serialize; open: Open; onOpen: (open: Open) => void }) {
  const session = useSession();
  const libraryBusy = busyIn(session, "library");
  const aiBusy = busyIn(session, "ai");
  const detail = card.detail;
  const filter = detail.documentsFilter;
  const [deleting, setDeleting] = useState(false);

  const end = async () => {
    if (!card.running) {
      await unwrap(commands.contractsEnd(card.id, "")).catch(reportError);
      return;
    }
    const endedOn = await dialogs.askText({ title: `End ${card.name}`, label: "Ended on (YYYY-MM-DD)", initialValue: localToday(), confirmLabel: "End contract" });
    if (endedOn !== null) await unwrap(commands.contractsEnd(card.id, endedOn)).catch(reportError);
  };

  const remove = async () => {
    const confirmed = await dialogs.confirm({
      title: `Delete ${card.name}`,
      message: "The contract leaves the catalog with its price history. Its documents stay in the library.",
      affected: [],
      confirmLabel: "Delete contract",
    });
    if (!confirmed) return;
    setDeleting(true);
    await unwrap(commands.contractsDelete(card.id, true)).catch(reportError);
    setDeleting(false);
  };

  return (
    <AppCard className="contract-detail">
      <div className="row gap-m">
        <LogoLetter letter={card.letter} colour={card.color} />
        <div className="column grow">
          <span className="title-medium ellipsis">{card.name}</span>
          <span className="body-small muted ellipsis">{detail.subtitle}</span>
        </div>
      </div>
      <Pills pills={card.pills} />
      <div className="contract-figures">
        <Figure label="Per month" value={detail.monthly} />
        <Figure label="Per year" value={detail.yearly} />
        <Figure label="Notice period" value={detail.notice} />
        <Figure label="Cancel by" value={detail.cancelBy} />
      </div>
      <div className="column gap-xs">
        <span className="body-small muted">Monthly price</span>
        <PriceChart points={detail.history} colour={chartColour(card.color)} />
      </div>
      {(detail.increase || detail.note) && (
        <p className="body-medium">
          {detail.increase && <span className="contract-pill warning">{detail.increase}</span>} {detail.note}
        </p>
      )}
      <div className="row gap-s wrap">
        {card.running && (
          <Button variant="filled" icon={EDIT_NOTE} onClick={() => onOpen("letter")} disabled={libraryBusy !== null} title={libraryBusy ? waitFor(libraryBusy) : undefined}>
            Draft cancellation letter
          </Button>
        )}
        {filter && <Button icon={FOLDER_OPEN} onClick={() => showDocumentsOf(filter)}>{detail.documentsLabel}</Button>}
        {session.aiEnabled && detail.documentIds.length > 0 && (
          <Button icon={AUTO_AWESOME} className="btn-ai" onClick={() => onOpen("terms")} disabled={aiBusy !== null} title={aiBusy ? waitFor(aiBusy) : undefined}>
            Read prices from documents
          </Button>
        )}
        <Button icon={EDIT} onClick={() => onOpen("editor")}>Edit</Button>
        <AddToEmergencyButton contractId={card.id} busyReason={libraryBusy ? waitFor(libraryBusy) : null} />
        <Button variant="text" onClick={() => void end()} disabled={libraryBusy !== null}>{card.running ? "End contract…" : "Runs again"}</Button>
        <Button variant="text" danger onClick={() => void remove()} disabled={deleting || libraryBusy !== null}>Delete…</Button>
      </div>
      {open === "letter" && (
        <LetterDialog kind="cancellation" documentIds={detail.documentIds} recipient={detail.recipient ?? undefined} subjectFacts={detail.letterFacts} onClose={() => onOpen(null)} />
      )}
      {open === "editor" && <ContractEditor initial={detail.editor} onClose={() => onOpen(null)} onSaved={() => onOpen(null)} />}
      {open === "terms" && <TermsReview contractId={card.id} name={card.name} onClose={() => onOpen(null)} />}
    </AppCard>
  );
}
