import {useQuery} from "@tanstack/react-query";
import {commands, describeError, unwrap} from "../../ipc";
import {LIBRARY_QUERY} from "../../session/session";
import {Button, Dialog, Spinner} from "../../shared/components";
import {markPaid} from "./queries";
import {SourcePill} from "./source-pill";

/** A bill's EPC QR code (GiroCode) with the values it holds, and "Mark as paid". */
export function PaymentDialog({ documentId, onClose }: { documentId: string; onClose: () => void }) {
  const code = useQuery({
    queryKey: [LIBRARY_QUERY, "bills", "code", documentId],
    queryFn: () => unwrap(commands.billsPaymentCode(documentId)),
  });
  const data = code.data;
  const buttons = (
    <>
      <Button variant="text" onClick={onClose}>Close</Button>
      {data && !data.paid && (
        <Button variant="filled" onClick={() => { markPaid(documentId); onClose(); }}>Mark as paid</Button>
      )}
    </>
  );
  return (
    <Dialog title={data ? `Pay “${data.title}”` : "Pay"} onDismiss={onClose} buttons={buttons} width={560}>
      {!data && (code.error ? <p className="body-medium">{describeError(code.error)}</p> : <Spinner />)}
      {data && (
        <div className="payment-code">
          {data.svg
            ? <div className="payment-qr" aria-label="Payment QR code" dangerouslySetInnerHTML={{ __html: data.svg }} />
            : <p className="payment-refusal body-medium">{data.refusal}</p>}
          <dl className="payment-values body-medium">
            <dt>Payee</dt><dd>{data.payee || "–"}</dd>
            <dt>IBAN</dt><dd className="mono">{data.iban ?? "–"}</dd>
            <dt>Amount</dt><dd><b>{data.amount}</b></dd>
            <dt>Reference</dt><dd className="mono">{data.reference ?? "–"}</dd>
            <dt>Due</dt><dd>{data.due}</dd>
            <dt>Data from</dt><dd><SourcePill source={data.source} /></dd>
          </dl>
        </div>
      )}
      {data?.svg && <p className="body-small muted">Scan with your banking app (GiroCode / EPC QR code) and check the values before you pay.</p>}
    </Dialog>
  );
}
