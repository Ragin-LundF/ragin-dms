import type {CasesFilterDto, CaseStatusPillDto, CasesViewDto} from "../../ipc";
import {CountPill} from "../../shared/components";
import {casesScreen} from "./state";

/** The segmented Active / Done switch and the cases of the chosen list with status and next step. */
export function CaseList({ view, filter, selected }: { view: CasesViewDto; filter: CasesFilterDto; selected: string | null }) {
  const choose = (next: CasesFilterDto) => casesScreen.set({ filter: next, selected: null });
  return (
    <div className="column gap-s case-list">
      <div className="case-segments" role="tablist">
        <Segment label="Active" count={view.activeCount} selected={filter === "active"} onClick={() => choose("active")} />
        <Segment label="Done" count={view.doneCount} selected={filter === "done"} onClick={() => choose("done")} />
      </div>
      {view.cases.map((row) => (
        <button
          key={row.id}
          type="button"
          className={row.id === selected ? "case-card selected" : "case-card"}
          onClick={() => casesScreen.set({ filter, selected: row.id })}
        >
          <span className="row gap-s">
            <span className="title-small grow ellipsis">{row.title}</span>
            <StatusPill pill={row.pill} />
          </span>
          {row.correspondent && <span className="body-small muted ellipsis">{row.correspondent}</span>}
          <span className="body-small ellipsis">→ {row.nextStep}</span>
        </button>
      ))}
    </div>
  );
}

function Segment({ label, count, selected, onClick }: { label: string; count: number; selected: boolean; onClick: () => void }) {
  return (
    <button type="button" role="tab" aria-selected={selected} className={selected ? "case-segment selected" : "case-segment"} onClick={onClick}>
      {label} <CountPill count={count} selected={selected} />
    </button>
  );
}

/** The status in a word; a due follow-up stands out. */
export function StatusPill({ pill }: { pill: CaseStatusPillDto }) {
  const tone = pill.attention ? "attention" : pill.status;
  return <span className={`case-pill ${tone}`}>{pill.label}</span>;
}
