import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import RECEIPT from "@material-design-icons/svg/round/receipt_long.svg?raw";
import {commands} from "../../../../ipc";
import {LIBRARY_QUERY} from "../../../../session/session";
import {Button, Icon, ReasonTooltip} from "../../../../shared/components";
import {TaxAssignDialog} from "../../../tax";
import type {OrganizeRowProps} from "./organize-block";

/** The document's tax year, area and deductible amount, with a button to assign or change it. */
export function TaxRow({ documentId, busyReason }: OrganizeRowProps) {
  const [editing, setEditing] = useState(false);
  const query = useQuery({ queryKey: [LIBRARY_QUERY, "tax-document", documentId], queryFn: () => commands.taxDocument(documentId) });
  const tax = query.data?.status === "ok" ? query.data.data : null;
  if (!tax) return null;
  return (
    <div className="row gap-s">
      <Icon svg={RECEIPT} size={18} className="muted" />
      <span className="body-medium grow">{tax.text ? `Tax: ${tax.text}` : <span className="muted">No tax year</span>}</span>
      <ReasonTooltip reason={busyReason}>
        <Button variant="text" small disabled={busyReason !== null} onClick={() => setEditing(true)}>{tax.text ? "Edit" : "Assign"}</Button>
      </ReasonTooltip>
      {editing && <TaxAssignDialog documentIds={[documentId]} onClose={() => setEditing(false)} />}
    </div>
  );
}
