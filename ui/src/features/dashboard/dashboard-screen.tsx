import {useCallback} from "react";
import {commands, describeError, unwrap} from "../../ipc";
import {DropOverlay, EmptyState, Spinner} from "../../shared/components";
import {goTo} from "../../shell/screens";
import {type LibrarySelection, openDocument, openLibrary} from "../library/navigation";
import {type DashboardView, parseYear, reportUnlessBusy, useDashboardView} from "./actions";
import {useFileDrop} from "./file-drop";
import {AttentionPanel} from "./overview/attention-panel";
import {AiBand, DashboardHeader, StatRow} from "./overview/header";
import {CategoryCard, CorrespondentCard, KindCard, TagCloudCard, YearChartCard} from "./overview/library-cards";
import {RecentDocumentsCard} from "./overview/recent-card";
import {ActivityPanel, FolderPanel} from "./overview/status-cards";
import ERROR from "@material-design-icons/svg/round/error_outline.svg?raw";

const show = (selection: LibrarySelection, query = "") => openLibrary({ kind: "show", selection, query });

/** The home screen; files dropped anywhere on it go to the top level (Kotlin `DashboardScreen`). */
export function DashboardScreen() {
  const view = useDashboardView();
  const onDrop = useCallback((paths: string[]) => void unwrap(commands.libraryAddFiles(paths, "")).catch(reportUnlessBusy), []);
  const hovering = useFileDrop(onDrop);
  return (
    <div className="dashboard-screen">
      {view.data ? <DashboardOverview view={view.data} /> : <Loading error={view.error} />}
      {hovering && <DropOverlay label="The files go to the top level of every library folder" />}
    </div>
  );
}

export function Loading({ error }: { error: unknown }) {
  if (!error) return <div className="center-fill"><Spinner /></div>;
  return <EmptyState icon={ERROR} title="The dashboard is not available" message={describeError(error)} style={{ height: "100%" }} />;
}

/** The sections one below the other; clicks open the library at what was clicked (Kotlin `DashboardOverview`). */
function DashboardOverview({ view }: { view: DashboardView }) {
  return (
    <div className="dashboard-overview">
      <DashboardHeader line={view.headerLine} />
      <StatRow columns={view.stats} onOpen={(link) => (link === "inbox" ? goTo("inbox") : show({ kind: "uncategorized" }))} />
      {view.aiBand && <AiBand band={view.aiBand} />}
      <div className="dashboard-section focus-section">
        <AttentionPanel attention={view.attention} onShowAllTasks={() => goTo("tasks")} />
        <RecentDocumentsCard documents={view.recent} primaryFolderId={view.primaryFolderId} onOpen={openDocument} onShowAll={() => show({ kind: "recent" })} />
      </div>
      <div className="dashboard-section quick-access">
        <CategoryCard
          points={view.categories}
          onOpenCategory={(categoryId) => show({ kind: "category", categoryId })}
          onOpenUncategorized={() => show({ kind: "uncategorized" })}
        />
        <TagCloudCard points={view.tags} onOpenTag={(tag) => show({ kind: "tag", tag })} />
        <YearChartCard points={view.byYear} onOpenYear={openYear} />
      </div>
      <div className="dashboard-section insights">
        <KindCard kinds={view.byKind} />
        <CorrespondentCard points={view.correspondents} onSearch={(name) => show({ kind: "folder", path: "" }, name)} />
      </div>
      <div className="dashboard-section status-section">
        <FolderPanel folders={view.folders} />
        <ActivityPanel entries={view.activity} />
      </div>
    </div>
  );
}

function openYear(id: string) {
  const year = parseYear(id);
  if (!year) return;
  openLibrary({ kind: "show", selection: { kind: "folder", path: "" }, query: "", year: year.year, yearAndEarlier: year.andEarlier });
}
