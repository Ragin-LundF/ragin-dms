import {useStore} from "../../shared/store";
import {LetterDialog} from "../letters";
import {billsOverlay} from "./overlay";
import {PaymentDialog} from "./payment-dialog";

/** The payment code and the request letter, opened from the dashboard, the "To pay" screen or the quick look. */
export function BillsDialogs() {
  const overlay = useStore(billsOverlay);
  const close = () => billsOverlay.set(null);
  if (!overlay) return null;
  if (overlay.kind === "paymentCode") return <PaymentDialog documentId={overlay.documentId} onClose={close} />;
  return (
    <LetterDialog
      kind="request"
      documentIds={overlay.documentIds}
      recipient={overlay.recipient}
      subjectFacts={overlay.subjectFacts}
      onClose={close}
      onSaved={close}
    />
  );
}
