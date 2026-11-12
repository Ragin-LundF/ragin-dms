import {AppCard} from "../../../shared/components";
import {goTo} from "../../../shell/screens";
import {useBillsView} from "../../bills";

/** The status card "To pay this month": unpaid bills due by the end of the month, and how many are overdue. */
export function ToPayCard() {
  const view = useBillsView("open").data;
  if (!view || (view.dueThisMonth.count === 0 && view.open.count === 0)) return null;
  return (
    <AppCard className="organize-card" onClick={() => goTo("bills")}>
      <div className="body-small muted">To pay this month</div>
      <div className="organize-stat">{view.dueThisMonth.amount || "0"}</div>
      {view.overdue.count > 0
        ? <span className="organize-pill tone-error">{view.overdue.count} overdue</span>
        : <span className="organize-pill tone-good">None overdue</span>}
    </AppCard>
  );
}
