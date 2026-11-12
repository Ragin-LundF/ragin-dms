import SEND from "@material-design-icons/svg/round/send.svg?raw";
import type {TaxYearViewDto_Serialize} from "../../ipc";
import {AppCard, Button, ProgressBar} from "../../shared/components";
import {openTaxAdviserExport} from "../library";

/** The side of the tax screen: progress with its legend, the amounts per area and "Send to tax adviser". */
export function TaxSummary({ view }: { view: TaxYearViewDto_Serialize }) {
  return (
    <div className="tax-side">
      <AppCard>
        <div className="column gap-s">
          <span className="body-small muted">Progress</span>
          <span className="tax-stat">{view.found} <span className="body-medium muted">of {view.total} found</span></span>
          <ProgressBar fraction={view.total === 0 ? 1 : view.found / view.total} />
          <div className="tax-legend body-small">
            <span className="tax-legend-found">found</span>
            <span className="tax-legend-expected">expected, not yet due</span>
            <span className="tax-legend-missing">missing</span>
          </div>
        </div>
      </AppCard>
      <AppCard>
        <div className="column gap-s">
          <span className="title-small">Amounts per area</span>
          {view.sums.length === 0 && <span className="body-small muted">No amounts yet. They come from the documents assigned to {view.year}.</span>}
          {view.sums.map((sum) => (
            <div key={sum.area.area} className="tax-sum">
              <div className="column">
                <span className="body-small">{sum.area.label}</span>
                <span className="label-small muted">{sum.area.form}</span>
              </div>
              <span className="label-large tax-amount">{sum.amounts.join(" + ")}</span>
            </div>
          ))}
        </div>
      </AppCard>
      <Button variant="filled" icon={SEND} onClick={() => openTaxAdviserExport(view.year)}>Send to tax adviser…</Button>
    </div>
  );
}
