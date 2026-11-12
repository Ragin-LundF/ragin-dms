import {useQuery} from "@tanstack/react-query";
import {commands, unwrap} from "../../../../ipc";
import {LIBRARY_QUERY, reportError} from "../../../../session/session";
import {Button, ReasonTooltip} from "../../../../shared/components";
import {markPaid, openPaymentCode} from "../../../bills";
import type {OrganizeRowProps} from "./organize-block";

/** The "Paid on" row of a bill, with "Mark as paid" and its payment code; PDFs that are no bill yet can read their e-invoice. */
export function BillRow({ documentId, busyReason }: OrganizeRowProps) {
  const bill = useQuery({
    queryKey: [LIBRARY_QUERY, "bills", "document", documentId],
    queryFn: () => unwrap(commands.billsDocument(documentId)),
  }).data;
  if (!bill) return null;
  if (!bill.isBill) {
    if (!bill.canReadEInvoice) return null;
    return (
      <div className="row gap-s quick-look-bill">
        <span className="body-small muted grow">E-invoice: exact bill values without the AI, if the PDF carries one</span>
        <ReasonTooltip reason={busyReason}>
          <Button small disabled={busyReason !== null} onClick={() => void unwrap(commands.billsReadEInvoice(documentId)).catch(reportError)}>
            Read e-invoice
          </Button>
        </ReasonTooltip>
      </div>
    );
  }
  return (
    <div className="row gap-s quick-look-bill">
      <span className="body-small muted">Paid on</span>
      <span className="body-medium grow">{bill.paidOn ?? (bill.status === "overdue" ? "Not paid, overdue" : "Not paid yet")}</span>
      {bill.paidOn === null && (
        <>
          <Button small variant="text" onClick={() => openPaymentCode(documentId)}>QR code</Button>
          <ReasonTooltip reason={busyReason}>
            <Button small disabled={busyReason !== null} onClick={() => markPaid(documentId)}>Mark as paid</Button>
          </ReasonTooltip>
        </>
      )}
    </div>
  );
}
