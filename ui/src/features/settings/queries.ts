import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { commands, unwrap, type AiDraftDto, type AiSettingsDto_Serialize } from "../../ipc";
import { LIBRARY_QUERY, busyIn, useSession } from "../../session/session";
import { waitFor } from "../../shared/components";

export function useSettingsState() {
  return useQuery({ queryKey: [LIBRARY_QUERY, "settings", "state"], queryFn: () => unwrap(commands.settingsState()) });
}

export function useLibrarySettings() {
  return useQuery({ queryKey: [LIBRARY_QUERY, "settings", "library"], queryFn: () => commands.librarySettings() });
}

/** The checked AI draft; keeps the last answer while the next one is on its way, so buttons do not flicker. */
export function useAiDraftCheck(draft: AiDraftDto, saved: AiSettingsDto_Serialize) {
  return useQuery({
    queryKey: ["settings", "ai-draft", draft, saved],
    queryFn: () => commands.checkAiDraft(draft, saved),
    placeholderData: keepPreviousData,
  });
}

export function useInboxPathProblem(path: string) {
  return useQuery({ queryKey: ["settings", "inbox-path", path], queryFn: () => commands.checkInboxPath(path), placeholderData: keepPreviousData });
}

/** Why every settings button waits: the operation running in the library lane, or null. */
export function useBusyReason(): string | null {
  const busy = busyIn(useSession(), "library");
  return busy === null ? null : waitFor(busy);
}
