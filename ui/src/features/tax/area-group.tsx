import ADD from "@material-design-icons/svg/round/add.svg?raw";
import CHEVRON from "@material-design-icons/svg/round/chevron_right.svg?raw";
import type {TaxAreaGroupDto_Serialize, TaxItemDto_Serialize} from "../../ipc";
import {Button, Icon} from "../../shared/components";
import {TaxItemRow} from "./item-row";
import {taxActions} from "./tax-actions";

const STATUS = {
  complete: { text: "complete", className: "tax-pill-good" },
  expected: { text: "expected", className: "tax-pill-warning" },
  missing: { text: "missing", className: "tax-pill-error" },
} as const;

/** One area of the return, folded open while something is missing or expected. */
export function TaxAreaGroup({ year, group, disabledReason, onHaveIt, onRequest }: {
  year: number;
  group: TaxAreaGroupDto_Serialize;
  disabledReason: string | null;
  onHaveIt: (item: TaxItemDto_Serialize) => void;
  onRequest: (item: TaxItemDto_Serialize) => void;
}) {
  const status = STATUS[group.status];
  return (
    <details className="tax-area" open={group.status !== "complete"}>
      <summary className="tax-area-head">
        <Icon svg={CHEVRON} className="tax-chevron" />
        <span className="title-small">{group.area.label}</span>
        <span className="body-small muted">{group.area.form}</span>
        <span className="body-small tax-count">{group.found} / {group.total}</span>
        <span className={`label-small tax-pill ${status.className}`}>{status.text}</span>
      </summary>
      <div className="tax-items">
        {group.items.map((item) => (
          <TaxItemRow key={item.id} year={year} item={item} disabledReason={disabledReason} onHaveIt={onHaveIt} onRequest={onRequest} />
        ))}
        <Button variant="text" small icon={ADD} disabled={disabledReason !== null} onClick={() => void taxActions.add(year, group.area.area, group.area.label)}>
          Add item
        </Button>
      </div>
    </details>
  );
}
