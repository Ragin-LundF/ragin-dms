import RULE from "@material-design-icons/svg/round/rule.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import type { RuleRowDto } from "../../../ipc";
import { AppCard, CardTitle, IconButton, Switch } from "../../../shared/components";
import { changeLibrarySettings } from "./change";

/**
 * The matching rules: what each one looks for and proposes, whether it was learned or written by hand, on or off.
 * New rules come from accepted proposals on the dashboard and from "Create rule from this document".
 */
export function RulesCard({ rules, busyReason }: { rules: RuleRowDto[]; busyReason: string | null }) {
  const enabled = busyReason === null;
  return (
    <AppCard className="settings-card">
      <CardTitle title="Matching rules" icon={RULE} tint="var(--tertiary)" subtitle="Values proposed without the AI; you still approve every suggestion" />
      {rules.length === 0 && (
        // "quickLookViewModel" (sic) is the Kotlin 1.3.0 wording, kept for parity.
        <span className="body-small muted">No rules yet. Accept a learned rule on the dashboard, or create one from a document in the quickLookViewModel.</span>
      )}
      {rules.map((rule) => (
        <div key={rule.id} className="row">
          <div className="column grow">
            <span className="title-small">{rule.title}</span>
            <span className="body-small muted settings-two-lines">{rule.description}</span>
          </div>
          <Switch label={rule.name} checked={rule.enabled} disabled={!enabled} onChange={(on) => changeLibrarySettings({ kind: "setRuleEnabled", ruleId: rule.id, enabled: on })} />
          <IconButton icon={DELETE} label={`Remove ${rule.name}`} disabled={!enabled} onClick={() => changeLibrarySettings({ kind: "removeRule", ruleId: rule.id })} />
        </div>
      ))}
    </AppCard>
  );
}
