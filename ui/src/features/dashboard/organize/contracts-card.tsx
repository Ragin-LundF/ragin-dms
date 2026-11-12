import {AppCard} from "../../../shared/components";
import {goTo} from "../../../shell/screens";
import {useContractsStatus} from "../../contracts";

/** The status card "Contracts per month"; hidden while no running contract has a price. */
export function ContractsCard() {
  const { data: status } = useContractsStatus();
  if (!status) return null;
  return (
    <AppCard className="organize-card" onClick={() => goTo("contracts")}>
      <div className="body-small muted">Contracts per month</div>
      {status.monthly.map((amount) => <div key={amount} className="organize-stat">{amount}</div>)}
      {status.increasesLabel && <span className="organize-pill tone-warning">{status.increasesLabel}</span>}
    </AppCard>
  );
}
