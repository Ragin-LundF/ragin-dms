import {useEffect, useState} from "react";
import {type BillsFilterDto, commands, unwrap} from "../../ipc";
import {busyIn, reportError, useSession} from "../../session/session";
import {
    AppCard,
    Button,
    dialogs,
    FoldToggle,
    ReasonTooltip,
    ScreenHeader,
    Spinner,
    waitFor
} from "../../shared/components";
import {BillsTable} from "./bills-table";
import {useBillsView} from "./queries";
import "./bills.css";

export { BillsDialogs } from "./bills-dialogs";
export { openPaymentCode, openRequestLetter } from "./overlay";
export { markPaid, useBillsView, useExpectedMissing } from "./queries";

const FILTERS: { filter: BillsFilterDto; label: string }[] = [
  { filter: "open", label: "Open" },
  { filter: "paid", label: "Paid" },
  { filter: "all", label: "All" },
];

/** "To pay": every bill that is not paid yet, by due date, with totals and the older bills never marked. */
export function BillsScreen() {
  const [filter, setFilter] = useState<BillsFilterDto>("open");
  const view = useBillsView(filter);
  const session = useSession();
  const busy = busyIn(session, "library");
  const busyReason = busy ? waitFor(busy) : null;
  const data = view.data;
  useEffect(() => {
    if (data && !data.trackingStarted) void unwrap(commands.billsStartTracking()).catch(reportError);
  }, [data]);
  return (
    <div className="screen">
      <div className="bills-screen">
        <ScreenHeader title="To pay" subtitle="Every bill that is not paid yet, by due date" />
        {!data ? <Spinner /> : (
          <>
            <div className="bills-totals">
              <Total label="Open" total={data.open} />
              <Total label="Overdue" total={data.overdue} tone="error" />
              <Total label={`Paid in ${data.month}`} total={data.paidThisMonth} tone="good" />
            </div>
            <AppCard className="bills-card">
              <div className="row gap-m bills-toolbar">
                <div className="segmented" role="tablist">
                  {FILTERS.map((entry) => (
                    <button key={entry.filter} role="tab" aria-selected={entry.filter === filter} className={entry.filter === filter ? "on" : ""} onClick={() => setFilter(entry.filter)}>
                      {entry.label}
                    </button>
                  ))}
                </div>
                <span className="body-small muted grow" style={{ textAlign: "right" }}>Sorted by due date</span>
              </div>
              <BillsTable rows={data.rows} busyReason={busyReason} empty="Nothing here." />
            </AppCard>
            {data.older.length > 0 && filter !== "paid" && <OlderBills rows={data.older} busyReason={busyReason} />}
          </>
        )}
      </div>
    </div>
  );
}

function Total({ label, total, tone }: { label: string; total: { amount: string; count: number }; tone?: "error" | "good" }) {
  return (
    <AppCard>
      <div className="body-small muted">{label}</div>
      <div className={`bills-total ${tone ? `tone-${tone}` : ""}`}>{total.amount || "0"}</div>
      <div className="body-small muted">{total.count === 1 ? "1 bill" : `${total.count} bills`}</div>
    </AppCard>
  );
}

/** Bills due before tracking started, folded, with "Mark as settled" for all of them after a question. */
function OlderBills({ rows, busyReason }: { rows: Parameters<typeof BillsTable>[0]["rows"]; busyReason: string | null }) {
  const [expanded, setExpanded] = useState(false);
  const settle = async () => {
    const confirmed = await dialogs.confirm({
      title: "Mark older bills as settled",
      message: "Each of these bills gets its due date as the day it was paid. Undo works per bill.",
      affected: rows.map((row) => `${row.title} · ${row.amount} · due ${row.due}`),
      confirmLabel: "Mark as settled",
    });
    if (confirmed) await unwrap(commands.billsSettleOlder()).catch(reportError);
  };
  return (
    <AppCard className="bills-card">
      <div className="row gap-m">
        <FoldToggle expanded={expanded} onToggle={() => setExpanded(!expanded)} />
        <span className="title-small grow">Older bills never marked ({rows.length})</span>
        <ReasonTooltip reason={busyReason}><Button disabled={busyReason !== null} onClick={() => void settle()}>Mark as settled…</Button></ReasonTooltip>
      </div>
      {expanded && <BillsTable rows={rows} busyReason={busyReason} empty="" />}
    </AppCard>
  );
}
