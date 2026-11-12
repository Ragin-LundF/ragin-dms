import "./dashboard.css";
import {DashboardScreen, Loading} from "./dashboard-screen";
import {useDashboardView} from "./actions";
import {TaskList} from "./tasks/task-list";

export { DashboardScreen };
export { DashButtons } from "./tasks/buttons";

/** Every open task, filling the screen; it shares the dashboard's filter (Kotlin `TasksScreen`). */
export function TasksScreen() {
  const view = useDashboardView();
  return <div className="tasks-screen">{view.data ? <TaskList groups={view.data.groups} /> : <Loading error={view.error} />}</div>;
}
