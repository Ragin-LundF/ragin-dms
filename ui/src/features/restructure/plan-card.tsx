import DRIVE_FILE_MOVE from "@material-design-icons/svg/round/drive_file_move.svg?raw";
import type {MoveGroupDto, PlanDto} from "../../ipc";
import {ActionButtons, AppCard, CardTitle} from "../../shared/components";

/**
 * The planned restructure: the new folders with how many documents go into each (untick a folder to leave its
 * documents where they are), what stays and why, the names that get a number, and Apply.
 */
export function PlanCard({ plan, excluded, busyReason, onToggle, onApply }: {
  plan: PlanDto;
  excluded: string[];
  busyReason: string | null;
  onToggle: (folder: string) => void;
  onApply: () => void;
}) {
  const included = plan.groups.filter((group) => !excluded.includes(group.folder));
  const moves = included.reduce((sum, group) => sum + group.count, 0);
  return (
    <AppCard className="restructure-card">
      <CardTitle
        title="Plan"
        icon={DRIVE_FILE_MOVE}
        tint="var(--secondary)"
        subtitle={`${moves} moves into ${included.length} folders · ${plan.unchanged} already in place`}
      />
      {plan.skipped.length > 0 && <Note text={`Left where they are: ${plan.skipped.join(", ")}`} />}
      {plan.renamed.length > 0 && <Note text={`Name taken, numbered: ${plan.renamed.join("; ")}`} />}
      <div className="restructure-groups scroll">
        {plan.groups.map((group) => (
          <GroupRow key={group.folder} group={group} included={!excluded.includes(group.folder)} onToggle={() => onToggle(group.folder)} />
        ))}
      </div>
      <ActionButtons
        enabled={busyReason === null}
        busyReason={busyReason}
        buttons={[{ label: `Apply ${moves} moves`, style: "filled", disabledReason: moves === 0 ? "Nothing to move" : null, onClick: onApply }]}
      />
    </AppCard>
  );
}

function GroupRow({ group, included, onToggle }: { group: MoveGroupDto; included: boolean; onToggle: () => void }) {
  const folder = group.folder === "" ? "Top level" : group.folder;
  return (
    <div className="row gap-s restructure-group">
      <input type="checkbox" className="checkbox" checked={included} onChange={onToggle} aria-label={folder} />
      <div className="column grow" style={{ minWidth: 0 }}>
        <span className="title-small ellipsis" style={{ whiteSpace: "pre" }}>{`${folder}  (${group.count})`}</span>
        {group.samples.map((sample) => (
          <span key={sample} className="body-small muted ellipsis">{sample}</span>
        ))}
      </div>
    </div>
  );
}

function Note({ text }: { text: string }) {
  return <span className="body-small muted">{text}</span>;
}
