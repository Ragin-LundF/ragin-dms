import type {TaskRowDto_Serialize} from "../../../ipc";
import {SectionHeader} from "../../../shared/components";
import {DashButtons} from "../../dashboard";

/** The open findings with the same buttons as on the dashboard (repair from a folder, dismiss). */
export function HealthFindings({ findings }: { findings: TaskRowDto_Serialize[] }) {
  return (
    <div className="column gap-s">
      <SectionHeader title="Findings" />
      {findings.length === 0 ? (
        <span className="body-medium muted">Nothing to repair.</span>
      ) : (
        findings.map((row) => (
          <div key={row.key} className="health-finding">
            <div className="column grow">
              <span className="title-small ellipsis">{row.title}</span>
              <span className="body-small muted">{row.detail}</span>
            </div>
            <DashButtons buttons={[...row.opens, ...row.buttons]} />
          </div>
        ))
      )}
    </div>
  );
}
