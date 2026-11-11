import {useEffect, useRef} from "react";
import {useQuery, useQueryClient} from "@tanstack/react-query";
import {commands, type RestructureStateDto_Serialize, unwrap} from "../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, useSession} from "../../session/session";
import {dialogs, ScreenHeader, waitFor} from "../../shared/components";
import {useStore} from "../../shared/store";
import {takeRequest, useNavigation} from "../../shell/screens";
import {restructureDrafts, type RestructureDrafts, sameScope, updateDrafts} from "./drafts";
import {HistoryCard} from "./history-card";
import {PlanCard} from "./plan-card";
import {TemplateCard} from "./template-card";
import "./restructure.css";

export { openRestructure } from "./navigation";

const STATE_QUERY = [LIBRARY_QUERY, "restructure", "state"];

/** The restructure assistant: template, plan and the restructures that can be reversed. */
export function RestructureScreen() {
  const session = useSession();
  const drafts = useStore(restructureDrafts);
  const queries = useQueryClient();
  const state = useQuery({ queryKey: STATE_QUERY, queryFn: () => unwrap(commands.restructureState()) });
  const history = useQuery({ queryKey: [LIBRARY_QUERY, "restructure", "history"], queryFn: () => unwrap(commands.restructureHistory()) });
  const busy = busyIn(session, "library");
  const busyReason = busy === null ? null : waitFor(busy);
  const data = state.data;

  // Kotlin refilled an empty template with the saved one (or the default) whenever the screen showed.
  useEffect(() => {
    if (data && restructureDrafts.get().template === "") updateDrafts({ template: data.initialTemplate });
  }, [data]);
  usePlanRequest(data, (drafts) => void preview(drafts));
  useEmptyFolderOffer(data?.emptyFolders ?? []);

  if (!data) return null;
  const scopeIndex = Math.max(0, data.scopes.findIndex((option) => sameScope(drafts.scope, option.scope)));

  async function preview(current: RestructureDrafts) {
    const scope = current.scope ?? data?.scopes[0]?.scope;
    if (!scope) return;
    try {
      const result = await unwrap(commands.previewRestructure(scope, current.template, current.naming));
      if (result.kind === "problem") {
        updateDrafts({ problem: result.text });
        return;
      }
      updateDrafts({ problem: null, planned: true, excluded: [] });
      await queries.invalidateQueries({ queryKey: STATE_QUERY });
    } catch (error) {
      reportError(error);
    }
  }

  const save = () => void unwrap(commands.savePathTemplate(drafts.template)).catch(reportError);

  const apply = async () => {
    const included = (data.plan?.groups ?? []).map((group) => group.folder).filter((folder) => !drafts.excluded.includes(folder));
    try {
      const chosen = await unwrap(commands.restructureApplyPreview(included));
      if (chosen.moves === 0) return;
      const confirmed = await dialogs.confirm({
        title: `Move ${chosen.moves} documents`,
        message: "The files move in every library folder. You can reverse this restructure later.",
        affected: chosen.affected,
        confirmLabel: "Move",
      });
      if (confirmed) await unwrap(commands.applyRestructure(included));
    } catch (error) {
      reportError(error);
    }
  };

  const reverse = async (name: string) => {
    const confirmed = await dialogs.confirm({
      title: `Reverse restructure ${name}`,
      message: "Every document still where this restructure put it goes back. Documents moved since stay.",
      affected: [name],
      confirmLabel: "Reverse",
    });
    if (confirmed) await unwrap(commands.reverseRestructure(name)).catch(reportError);
  };

  const toggle = (folder: string) => {
    const excluded = drafts.excluded.includes(folder) ? drafts.excluded.filter((entry) => entry !== folder) : [...drafts.excluded, folder];
    updateDrafts({ excluded });
  };

  return (
    <div className="screen">
      <div className="restructure-screen">
        <ScreenHeader title="Restructure" subtitle="Sort documents into folders by a template, after you approve" />
        <TemplateCard state={data} drafts={drafts} scopeIndex={scopeIndex} busyReason={busyReason} onPreview={() => void preview(drafts)} onSave={save} />
        {drafts.planned && data.plan && <PlanCard plan={data.plan} excluded={drafts.excluded} busyReason={busyReason} onToggle={toggle} onApply={() => void apply()} />}
        {(history.data ?? []).length > 0 && <HistoryCard history={history.data ?? []} busyReason={busyReason} onReverse={(name) => void reverse(name)} />}
      </div>
    </div>
  );
}

/**
 * "Restructure…" on a dashboard task opens the screen on a folder: once the scopes are known, that folder becomes
 * the scope and the plan is previewed at once (Kotlin `RestructureViewModel.start`).
 */
function usePlanRequest(state: RestructureStateDto_Serialize | undefined, preview: (drafts: RestructureDrafts) => void) {
  const nav = useNavigation();
  const pending = useRef<string | null>(null);
  useEffect(() => {
    const request = takeRequest<{ folder: string }>("restructure");
    if (request) pending.current = request.folder;
  }, [nav.request]);
  useEffect(() => {
    if (!state || pending.current === null) return;
    const folder = pending.current;
    pending.current = null;
    const option = state.scopes.find((candidate) => candidate.scope.kind === "folder" && candidate.scope.path === folder);
    if (!option) return;
    updateDrafts({ scope: option.scope, planned: false });
    preview(restructureDrafts.get());
  });
}

/** After a restructure, the folders it left empty are offered for removal, once. */
function useEmptyFolderOffer(folders: string[]) {
  const asking = useRef(false);
  const key = folders.join("\n");
  useEffect(() => {
    if (folders.length === 0 || asking.current) return;
    asking.current = true;
    void dialogs
      .confirm({
        title: "Remove empty folders",
        message: "These folders hold no documents any more. They go to the trash in every library folder.",
        affected: folders,
        confirmLabel: "Move to trash",
      })
      .then((confirmed) => (confirmed ? unwrap(commands.removeEmptyFolders(folders)) : commands.dismissEmptyFolders()))
      .catch(reportError)
      .finally(() => (asking.current = false));
  }, [key]); // the list's content, not its identity, decides
}
