import {useEffect, useState} from "react";
import ADD from "@material-design-icons/svg/round/add.svg?raw";
import ICON from "@material-design-icons/svg/round/autorenew.svg?raw";
import {commands, type ContractEditorDto} from "../../ipc";
import {reportError} from "../../session/session";
import {AppCard, Button, EmptyState, ScreenHeader, Spinner} from "../../shared/components";
import {takeRequest, useNavigation} from "../../shell/screens";
import {ContractCard} from "./contract-card";
import {ContractDetail} from "./contract-detail";
import {ContractEditor} from "./contract-editor";
import {type ContractsRequest, useContractsView} from "./contracts";
import "./contracts.css";

export { createContract, openContract, attachToContract, useContractsOfDocument, useContractsStatus } from "./contracts";

type Open = "letter" | "editor" | "terms" | null;

/** Contracts and subscriptions: the totals, one card per contract, and the selected contract's details. */
export function ContractsScreen() {
  const { data: view, isLoading } = useContractsView();
  const request = useNavigation().request;
  const [selected, setSelected] = useState<string | null>(null);
  const [open, setOpen] = useState<Open>(null);
  const [creating, setCreating] = useState<ContractEditorDto | null>(null);

  useEffect(() => {
    if (request?.screen !== "contracts") return;
    const taken = takeRequest<ContractsRequest>("contracts");
    if (taken?.kind === "open") {
      setSelected(taken.contractId);
      setOpen(taken.draftLetter ? "letter" : null);
    } else if (taken?.kind === "create") {
      commands.contractsNewForm(taken.correspondent, taken.documentIds).then(setCreating, reportError);
    }
  }, [request]);

  const newContract = () => commands.contractsNewForm("", []).then(setCreating, reportError);
  const cards = view?.cards ?? [];
  const card = cards.find((entry) => entry.id === selected) ?? cards[0] ?? null;

  return (
    <div className="screen">
      <div className="contracts-screen">
        <ScreenHeader
          title="Contracts"
          subtitle="What runs, what it costs, and by when it can be cancelled"
          actions={<Button variant="filled" icon={ADD} onClick={() => void newContract()}>New contract</Button>}
        />
        {isLoading && <Spinner />}
        {view && cards.length === 0 && (
          <EmptyState icon={ICON} title="No contracts yet" message="Record a contract, or create one from a proposal on the dashboard." style={{ flex: 1 }} />
        )}
        {card && (
          <div className="contracts-layout">
            <div className="column gap-l">
              {view?.totals.map((total) => (
                <AppCard key={total.monthly} className="contracts-totals">
                  <div className="column">
                    <span className="body-small muted">All contracts</span>
                    <span className="headline-small">{total.monthly} <span className="body-medium muted">per month</span></span>
                  </div>
                  <div className="column">
                    <span className="body-small muted">Per year</span>
                    <span className="title-large">{total.yearly}</span>
                  </div>
                  {total.change && <span className={`contract-pill ${total.rising ? "warning" : "good"} contracts-change`}>{total.change}</span>}
                </AppCard>
              ))}
              <div className="contracts-grid">
                {cards.map((entry) => (
                  <ContractCard key={entry.id} card={entry} selected={entry.id === card.id} onSelect={() => setSelected(entry.id)} />
                ))}
              </div>
            </div>
            <ContractDetail key={card.id} card={card} open={open} onOpen={setOpen} />
          </div>
        )}
      </div>
      {creating && (
        <ContractEditor
          initial={creating}
          onClose={() => setCreating(null)}
          onSaved={(id) => {
            setCreating(null);
            setSelected(id);
          }}
        />
      )}
    </div>
  );
}
