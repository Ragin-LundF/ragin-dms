import {useQuery} from "@tanstack/react-query";
import {commands} from "../../../ipc";
import {LIBRARY_QUERY} from "../../../session/session";
import {AppCard, ProgressBar} from "../../../shared/components";
import {goTo} from "../../../shell/screens";

/** The status card "Tax year progress": how much of the default tax year's checklist is found; opens the tax screen. */
export function TaxCard() {
  const query = useQuery({ queryKey: [LIBRARY_QUERY, "tax-progress"], queryFn: () => commands.taxProgress() });
  const progress = query.data?.status === "ok" ? query.data.data : null;
  if (!progress) return null;
  return (
    <AppCard onClick={() => goTo("tax")}>
      <div className="column gap-xs">
        <span className="body-small muted">Tax year {progress.year}</span>
        <span className="title-large">{progress.found} <span className="body-medium muted">/ {progress.total} found</span></span>
        <ProgressBar fraction={progress.total === 0 ? 1 : progress.found / progress.total} />
        {progress.missing > 0 && <span className="body-small muted">{progress.missing} missing</span>}
      </div>
    </AppCard>
  );
}
