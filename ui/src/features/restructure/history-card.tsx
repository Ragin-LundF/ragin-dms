import HISTORY from "@material-design-icons/svg/round/history.svg?raw";
import {ActionButtons, AppCard, CardTitle} from "../../shared/components";

const HISTORY_SHOWN = 5;

/** The applied restructures, newest first, each with a button to reverse it. */
export function HistoryCard({ history, busyReason, onReverse }: { history: string[]; busyReason: string | null; onReverse: (name: string) => void }) {
  return (
    <AppCard className="restructure-card">
      <CardTitle title="Applied restructures" icon={HISTORY} tint="var(--tertiary)" subtitle="Reverse puts back every document that is still where a restructure put it" />
      <ActionButtons
        enabled={busyReason === null}
        busyReason={busyReason}
        buttons={history.slice(0, HISTORY_SHOWN).map((name) => ({ label: `Reverse ${name}`, onClick: () => onReverse(name) }))}
      />
    </AppCard>
  );
}
