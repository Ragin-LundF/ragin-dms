import {commands, unwrap} from "../../../ipc";
import {AppCard, Button, CountPill} from "../../../shared/components";
import {openRequestLetter, useExpectedMissing} from "../../bills";
import {reportUnlessBusy} from "../actions";

/** "Expected but missing": documents a sender sends regularly that are late, with the rhythm in words. */
export function ExpectedCard() {
  const expected = useExpectedMissing().data ?? [];
  if (expected.length === 0) return null;
  const run = (result: ReturnType<typeof commands.billsExpectedRequested>) => void unwrap(result).catch(reportUnlessBusy);
  return (
    <div className="dashboard-section">
      <AppCard className="expected-card">
        <div className="row gap-s">
          <span className="title-medium">Expected but missing</span>
          <CountPill count={expected.length} />
        </div>
        <span className="body-small muted">Learned from the dates of earlier documents from the same sender.</span>
        {expected.map((item) => (
          <div key={item.taskKey} className="expected-row">
            <div className="column grow">
              <span className="title-small ellipsis">{item.title} <span className="muted">{item.correspondent}</span></span>
              <span className="body-small muted">{item.rhythm}. Expected around {item.expectedOn}.</span>
            </div>
            <Button small onClick={() => run(commands.billsExpectedRequested(item.taskKey))}>Requested</Button>
            <Button small variant="text" onClick={() => run(commands.runDashboardAction({ kind: "dismiss", taskKey: item.taskKey }))}>Ignore once</Button>
            <Button
              small
              variant="text"
              onClick={() => openRequestLetter(item.correspondent, item.documentIds, [`${item.title} expected around ${item.expectedOn}, not received`])}
            >
              Draft request
            </Button>
          </div>
        ))}
      </AppCard>
    </div>
  );
}
