import {BillRow} from "./bill-row";
import {CaseRow} from "./case-row";
import {ContractRow} from "./contract-row";
import {EmergencyRow} from "./emergency-row";
import {TaxRow} from "./tax-row";

export type OrganizeRowProps = { documentId: string; busyReason: string | null };

/** What the "Organize" features know about the shown document, one row per feature; hidden while none has a row. */
export function OrganizeBlock(props: OrganizeRowProps) {
  return (
    <div className="quick-look-organize">
      <BillRow {...props} />
      <ContractRow {...props} />
      <TaxRow {...props} />
      <CaseRow {...props} />
      <EmergencyRow {...props} />
    </div>
  );
}
