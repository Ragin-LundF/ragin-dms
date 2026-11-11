import TASK_ALT from "@material-design-icons/svg/round/task_alt.svg?raw";
import type {TaskGroupDto_Serialize, TaskRowDto_Serialize} from "../../../ipc";
import {
    AppCard,
    CardTitle,
    Chip,
    EmptyState,
    FileThumbnail,
    IconBadge,
    SectionHeader
} from "../../../shared/components";
import {useStore} from "../../../shared/store";
import {runButton, taskFilter} from "../actions";
import {DashButtons, DuplicateCopies, firstOpen} from "./buttons";
import {iconOf} from "./icons";

/** Every open task, grouped by kind, with a filter chip per kind (Kotlin `TaskList` on the tasks screen). */
export function TaskList({ groups }: { groups: TaskGroupDto_Serialize[] }) {
  const filter = useStore(taskFilter);
  const total = groups.reduce((sum, group) => sum + group.rows.length, 0);
  return (
    <AppCard className="task-list">
      <CardTitle
        title="Open tasks"
        icon={TASK_ALT}
        tint="var(--primary)"
        subtitle={total === 0 ? "Nothing waits for you" : `${total} waiting for your decision`}
      />
      {total === 0 ? (
        <EmptyState icon={TASK_ALT} title="All caught up" message="New, missing and changed files appear here after a scan." style={{ flex: 1 }} />
      ) : (
        <>
          <div className="row gap-s task-chips">
            <Chip label={`All (${total})`} selected={filter === null} onClick={() => taskFilter.set(null)} />
            {groups.map((group) => (
              <Chip key={group.group} label={`${group.label} (${group.rows.length})`} selected={filter === group.group} onClick={() => taskFilter.set(group.group)} />
            ))}
          </div>
          <div className="column gap-s scroll grow task-groups">
            {groups
              .filter((group) => filter === null || group.group === filter)
              .map((group) => (
                <div key={group.group} className="column gap-s">
                  <SectionHeader title={group.label} actions={<DashButtons buttons={group.bulkActions} />} />
                  {group.rows.map((row) => (
                    <TaskRow key={row.key} row={row} icon={group.icon} />
                  ))}
                </div>
              ))}
          </div>
        </>
      )}
    </AppCard>
  );
}

/** One task: thumbnail (or the group's icon), title with its open buttons, detail, copies and actions (Kotlin `TaskRowView`). */
function TaskRow({ row, icon }: { row: TaskRowDto_Serialize; icon: string }) {
  const badge = <IconBadge icon={iconOf(icon) ?? TASK_ALT} tint="var(--primary)" size={36} />;
  const open = firstOpen(row.opens, row.copies);
  return (
    <div className="task-row">
      {row.preview ? (
        <FileThumbnail folderId={row.preview.folderId} path={row.preview.path} onClick={() => open && void runButton(open)} fallback={badge} />
      ) : (
        badge
      )}
      <div className="column grow">
        <div className="row">
          <span className="title-small ellipsis grow">{row.title}</span>
          <DashButtons buttons={row.opens} />
        </div>
        {row.detail && <span className="body-small muted clamp-2">{row.detail}</span>}
        <DuplicateCopies copies={row.copies} />
        <DashButtons buttons={row.buttons} className="task-actions" />
      </div>
    </div>
  );
}
