import type {ContractCardDto_Serialize, ContractPillDto} from "../../ipc";
import {chartColour} from "./price-chart";

export function Pills({ pills }: { pills: ContractPillDto[] }) {
  return (
    <div className="row gap-xs wrap">
      {pills.map((pill) => (
        <span key={pill.label} className={`contract-pill ${pill.tone} nowrap`}>{pill.label}</span>
      ))}
    </div>
  );
}

export function LogoLetter({ letter, colour }: { letter: string; colour: number }) {
  return <span className="contract-logo" style={{ background: chartColour(colour) }} aria-hidden="true">{letter}</span>;
}

/** One contract in the grid: logo letter, name, price per month and its pills. */
export function ContractCard({ card, selected, onSelect }: { card: ContractCardDto_Serialize; selected: boolean; onSelect: () => void }) {
  return (
    <button type="button" className={`contract-card${selected ? " selected" : ""}${card.running ? "" : " ended"}`} onClick={onSelect} aria-pressed={selected}>
      <div className="row gap-m">
        <LogoLetter letter={card.letter} colour={card.color} />
        <div className="column grow">
          <span className="title-small ellipsis">{card.name}</span>
          {card.correspondent && <span className="body-small muted ellipsis">{card.correspondent}</span>}
        </div>
      </div>
      <div className="row gap-xs">
        <span className="title-medium">{card.monthly ?? "No price"}</span>
        {card.monthly && <span className="body-small muted">/ month</span>}
      </div>
      <Pills pills={card.pills} />
    </button>
  );
}
