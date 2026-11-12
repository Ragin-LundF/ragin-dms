import {useState} from "react";
import {useQuery} from "@tanstack/react-query";
import {commands, type TaxAreaDto, type TaxDocumentDto_Serialize} from "../../ipc";
import {LIBRARY_QUERY} from "../../session/session";
import {Button, Chip, Dialog, TextField} from "../../shared/components";
import {taxActions} from "./tax-actions";

/**
 * Assigns documents to a tax year and area. One document (the quick look) starts with its assignment and can get a
 * deductible part; many (the bulk bar) start empty and each keeps its own deductible part. "Remove" takes the
 * assignment away.
 */
export function TaxAssignDialog({ documentIds, onClose }: { documentIds: string[]; onClose: () => void }) {
  const first = documentIds[0] ?? "";
  const query = useQuery({ queryKey: [LIBRARY_QUERY, "tax-document", first], queryFn: () => commands.taxDocument(first) });
  const known = query.data?.status === "ok" ? query.data.data : null;
  if (!known) return null;
  return <AssignForm documentIds={documentIds} known={known} onClose={onClose} />;
}

function AssignForm({ documentIds, known, onClose }: { documentIds: string[]; known: TaxDocumentDto_Serialize; onClose: () => void }) {
  const single = documentIds.length === 1;
  const [year, setYear] = useState(String((single ? known.year : null) ?? known.defaultYear));
  const [area, setArea] = useState<TaxAreaDto | null>(single ? known.area : null);
  const [deductible, setDeductible] = useState(single ? known.deductible ?? "" : "");
  const parsedYear = /^\d{4}$/.test(year.trim()) ? Number(year.trim()) : null;
  const save = () => {
    if (parsedYear === null || area === null) return;
    void taxActions.assign(documentIds, parsedYear, area, single ? deductible : null, !single).then((done) => done && onClose());
  };
  const remove = () => void taxActions.assign(documentIds, null, null, null, true).then((done) => done && onClose());
  return (
    <Dialog
      title={single ? "Tax year and area" : `Tax year and area of ${documentIds.length} documents`}
      width={560}
      onDismiss={onClose}
      buttons={
        <>
          {(!single || known.year !== null) && <Button variant="text" danger onClick={remove}>Remove</Button>}
          <Button variant="text" onClick={onClose}>Cancel</Button>
          <Button variant="filled" disabled={parsedYear === null || area === null} onClick={save}>Assign</Button>
        </>
      }
    >
      <div className="column gap-m">
        <TextField label="Tax year" value={year} onChange={setYear} error={parsedYear === null ? "A year such as 2025" : undefined} labelBackground="var(--surface-high)" />
        <div className="row wrap gap-s" role="radiogroup" aria-label="Tax area">
          {known.areaOptions.map((option) => (
            <Chip key={option.area} label={option.label} title={option.form} selected={area === option.area} onClick={() => setArea(option.area)} />
          ))}
        </div>
        {single && area === "householdServices" && (
          <TextField
            label="Deductible part (labour share), if it differs from the amount"
            value={deductible}
            onChange={setDeductible}
            placeholder="684.50 EUR"
            labelBackground="var(--surface-high)"
          />
        )}
      </div>
    </Dialog>
  );
}
