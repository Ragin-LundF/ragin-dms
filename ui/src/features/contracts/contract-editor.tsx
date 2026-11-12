import {useState} from "react";
import {
    commands,
    type ContractEditorDto,
    type ContractKindDto,
    type ContractNoticeUnitDto,
    type ContractPeriodDto,
    describeError,
    unwrap
} from "../../ipc";
import {Button, Chip, Dialog, Switch, TextField} from "../../shared/components";

const KINDS: [ContractKindDto, string][] = [
  ["utility", "Utility"],
  ["telecom", "Telecom"],
  ["insurance", "Insurance"],
  ["subscription", "Subscription"],
  ["membership", "Membership"],
  ["loan", "Loan"],
  ["other", "Other"],
];
const PERIODS: [ContractPeriodDto, string][] = [
  ["monthly", "Per month"],
  ["quarterly", "Per quarter"],
  ["halfYearly", "Per half year"],
  ["yearly", "Per year"],
];
const UNITS: [ContractNoticeUnitDto, string][] = [
  ["days", "Days"],
  ["weeks", "Weeks"],
  ["months", "Months"],
];

function Choices<T extends string>({ label, options, value, onChange }: { label: string; options: [T, string][]; value: T; onChange: (value: T) => void }) {
  return (
    <div className="column gap-xs" role="radiogroup" aria-label={label}>
      <span className="label-medium muted">{label}</span>
      <div className="row gap-xs wrap">
        {options.map(([option, text]) => (
          <Chip key={option} label={text} selected={value === option} onClick={() => onChange(option)} />
        ))}
      </div>
    </div>
  );
}

/** Records a new contract or edits one. Rust checks every typed value and names the one that does not fit. */
export function ContractEditor({ initial, onClose, onSaved }: { initial: ContractEditorDto; onClose: () => void; onSaved: (contractId: string) => void }) {
  const [form, setForm] = useState(initial);
  const [problem, setProblem] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const set = <K extends keyof ContractEditorDto>(key: K) => (value: ContractEditorDto[K]) => setForm((current) => ({ ...current, [key]: value }));

  const save = () => {
    setSaving(true);
    unwrap(commands.contractsSave(form))
      .then(onSaved, (error: unknown) => setProblem(describeError(error)))
      .finally(() => setSaving(false));
  };

  return (
    <Dialog
      title={initial.id ? "Edit contract" : "New contract"}
      onDismiss={onClose}
      width={560}
      buttons={
        <>
          <Button variant="text" onClick={onClose}>Cancel</Button>
          <Button variant="filled" onClick={save} disabled={saving || form.name.trim() === ""}>Save</Button>
        </>
      }
    >
      <div className="column gap-l contract-editor">
        <div className="row gap-m">
          <TextField label="Name" value={form.name} onChange={set("name")} autoFocus style={{ flex: 1 }} />
          <TextField label="Correspondent" value={form.correspondent} onChange={set("correspondent")} style={{ flex: 1 }} />
        </div>
        <Choices label="Kind" options={KINDS} value={form.kind} onChange={set("kind")} />
        <div className="row gap-m">
          <TextField label="Price" value={form.price} onChange={set("price")} placeholder="44.99 EUR" style={{ flex: 1 }} />
          <TextField label="Price valid from" value={form.priceValidFrom} onChange={set("priceValidFrom")} placeholder="YYYY-MM-DD" support="Keeps the old price in the history" style={{ flex: 1 }} />
        </div>
        <Choices label="Charged" options={PERIODS} value={form.period} onChange={set("period")} />
        <div className="row gap-l">
          <span className="body-medium grow">Cancellable any time</span>
          <Switch label="Cancellable any time" checked={form.cancellableAnyTime} onChange={set("cancellableAnyTime")} />
        </div>
        {!form.cancellableAnyTime && (
          <>
            <div className="row gap-m">
              <TextField label="Term ends" value={form.termEnd} onChange={set("termEnd")} placeholder="YYYY-MM-DD" style={{ flex: 1 }} />
              <TextField label="Renews by (months)" value={form.renewsByMonths} onChange={set("renewsByMonths")} placeholder="12" style={{ flex: 1 }} />
            </div>
            <div className="row gap-m">
              <TextField label="Notice period" value={form.noticeCount} onChange={set("noticeCount")} placeholder="3" style={{ width: 140 }} />
              <Choices label="Unit" options={UNITS} value={form.noticeUnit} onChange={set("noticeUnit")} />
            </div>
          </>
        )}
        <TextField label="Note" value={form.note} onChange={set("note")} multiline rows={2} />
        {problem && <p className="body-small error-text" role="alert">{problem}</p>}
      </div>
    </Dialog>
  );
}
