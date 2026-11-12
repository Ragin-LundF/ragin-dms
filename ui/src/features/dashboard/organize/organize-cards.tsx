import {ContractsCard} from "./contracts-card";
import {HealthCard} from "./health-card";
import {TaxCard} from "./tax-card";
import {ToPayCard} from "./to-pay-card";

/** The status cards of the "Organize" features in one row; hidden while none has anything to show. */
export function OrganizeCards() {
  return (
    <div className="dashboard-section organize-cards">
      <ToPayCard />
      <ContractsCard />
      <TaxCard />
      <HealthCard />
    </div>
  );
}
