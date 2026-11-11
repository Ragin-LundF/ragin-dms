import type {FileNamingDto, RestructureScopeDto} from "../../ipc";
import {createStore} from "../../shared/store";

/**
 * What the user has typed and chosen on the restructure screen. It outlives the screen, as Kotlin's view model did:
 * leaving and coming back keeps the template, the scope and the plan's unticked folders.
 */
export type RestructureDrafts = {
  scope: RestructureScopeDto | null;
  template: string;
  naming: FileNamingDto;
  problem: string | null;
  planned: boolean;
  excluded: string[];
};

export const restructureDrafts = createStore<RestructureDrafts>({ scope: null, template: "", naming: "keep", problem: null, planned: false, excluded: [] });

export function updateDrafts(change: Partial<RestructureDrafts>) {
  restructureDrafts.set((drafts) => ({ ...drafts, ...change }));
}

/** Appends `{token}` as a new folder level. */
export function withPlaceholder(template: string, token: string): string {
  const separator = template === "" || template.endsWith("/") ? "" : "/";
  return `${template}${separator}{${token}}`;
}

export function sameScope(first: RestructureScopeDto | null, second: RestructureScopeDto): boolean {
  if (!first || first.kind !== second.kind) return false;
  return first.kind === "folder" ? first.path === (second as { path: string }).path : first.id === (second as { id: string }).id;
}
