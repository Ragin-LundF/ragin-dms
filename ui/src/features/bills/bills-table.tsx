import type {BillRowDto_Serialize} from "../../ipc";
import {Button, ReasonTooltip} from "../../shared/components";
import {openDocument} from "../library/navigation";
import {openPaymentCode} from "./overlay";
import {markPaid, markUnpaid} from "./queries";
import {SourcePill} from "./source-pill";

/** The bills by due date: bill, due pill, amount, where the data came from, and QR code / Paid / Undo. */
export function BillsTable({ rows, busyReason, empty }: { rows: BillRowDto_Serialize[]; busyReason: string | null; empty: string }) {
  return (
    <table className="bills-table">
      <thead>
        <tr><th>Bill</th><th>Due</th><th className="num">Amount</th><th>Data from</th><th /></tr>
      </thead>
      <tbody>
        {rows.length === 0 && <tr><td colSpan={5} className="bills-empty body-medium muted">{empty}</td></tr>}
        {rows.map((row) => (
          <tr key={row.documentId}>
            <td>
              <button className="bill-name" onClick={() => openDocument(row.documentId)}>
                <span className="title-small ellipsis">{row.title}</span>
                <span className="body-small muted ellipsis">{row.payee}</span>
              </button>
            </td>
            <td><DuePill row={row} /></td>
            <td className="num"><b>{row.amount}</b></td>
            <td><SourcePill source={row.source} /></td>
            <td className="num bill-actions">
              {row.status === "paid" ? (
                <ReasonTooltip reason={busyReason}><Button variant="text" small disabled={busyReason !== null} onClick={() => markUnpaid(row.documentId)}>Undo</Button></ReasonTooltip>
              ) : (
                <>
                  <ReasonTooltip reason={row.qrRefusal}><Button small disabled={row.qrRefusal !== null} onClick={() => openPaymentCode(row.documentId)}>QR code</Button></ReasonTooltip>
                  <ReasonTooltip reason={busyReason}><Button variant="filled" small disabled={busyReason !== null} onClick={() => markPaid(row.documentId)}>Paid</Button></ReasonTooltip>
                </>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function DuePill({ row }: { row: BillRowDto_Serialize }) {
  if (row.status === "paid") return <span className="bill-pill tone-green">Paid {row.paidOn ?? ""}</span>;
  if (row.dueInDays < 0) return <span className="bill-pill tone-red" title={row.due}>Overdue {-row.dueInDays} d</span>;
  if (row.dueInDays === 0) return <span className="bill-pill tone-amber" title={row.due}>Due today</span>;
  if (row.dueInDays <= 7) return <span className="bill-pill tone-amber" title={row.due}>In {row.dueInDays} days</span>;
  return <span className="bill-pill tone-grey">{row.due}</span>;
}
