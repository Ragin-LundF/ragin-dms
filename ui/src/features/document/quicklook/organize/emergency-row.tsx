import {AddToEmergencyButton} from "../../../emergency";
import type {OrganizeRowProps} from "./organize-block";

/** Whether the document is in the emergency folder, or "Add to emergency folder". */
export function EmergencyRow({ documentId, busyReason }: OrganizeRowProps) {
  return <AddToEmergencyButton documentId={documentId} busyReason={busyReason} />;
}
