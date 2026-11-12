import CHECK from "@material-design-icons/svg/round/check.svg?raw";
import type {TaxItemDto_Serialize} from "../../ipc";
import {type ActionButton, ActionButtons, ActionMenu, Icon} from "../../shared/components";
import {openQuickLook} from "../document";
import {taxActions} from "./tax-actions";

const STATE_LABELS = { found: "Found", expected: "Expected, not yet due", missing: "Missing", notNeeded: "Not needed this year" } as const;

/** One item of the checklist: its state mark, name and detail, the amount, and what can be done about it. */
export function TaxItemRow({ year, item, disabledReason, onHaveIt, onRequest }: {
  year: number;
  item: TaxItemDto_Serialize;
  disabledReason: string | null;
  onHaveIt: (item: TaxItemDto_Serialize) => void;
  onRequest: (item: TaxItemDto_Serialize) => void;
}) {
  const open = item.state === "missing" || item.state === "expected";
  const buttons: ActionButton[] = open
    ? [
        { label: "I have it", style: "outlined", disabledReason, onClick: () => onHaveIt(item) },
        { label: "Request", style: "text", disabledReason, onClick: () => onRequest(item) },
      ]
    : item.documentId
      ? [{ label: "Open", style: "text", onClick: () => item.documentId && openQuickLook(item.documentId, [item.documentId]) }]
      : [];
  const more: ActionButton[] = [
    { label: "Rename…", disabledReason, onClick: () => void taxActions.rename(year, item.id, item.label) },
    item.state === "notNeeded"
      ? { label: "Needed again", disabledReason, onClick: () => taxActions.change(year, { kind: "setNotNeeded", itemId: item.id, notNeeded: false }) }
      : { label: "Not needed this year", disabledReason, onClick: () => taxActions.change(year, { kind: "setNotNeeded", itemId: item.id, notNeeded: true }) },
  ];
  return (
    <div className={item.state === "notNeeded" ? "tax-item tax-item-off" : "tax-item"}>
      <span className={`tax-check tax-check-${item.state}`} title={STATE_LABELS[item.state]} aria-label={STATE_LABELS[item.state]}>
        {item.state === "found" && <Icon svg={CHECK} size={14} />}
      </span>
      <div className="tax-item-text">
        <span className="body-medium tax-ellipsis">{item.label}</span>
        {item.detail && <span className="body-small muted tax-ellipsis">{item.detail}</span>}
        {item.state === "notNeeded" && <span className="body-small muted">Not needed this year</span>}
      </div>
      {item.amount && <span className="body-small tax-amount">{item.amount}</span>}
      <ActionButtons buttons={buttons} />
      <ActionMenu actions={more} label={`More for ${item.label}`} />
    </div>
  );
}
