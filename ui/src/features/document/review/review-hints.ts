/**
 * Whether approving the typed category creates one, because no path and no last name of `paths` matches it ignoring
 * case (Kotlin `ReviewDrafts.isNewCategory`).
 */
export function isNewCategory(typed: string, paths: string[]): boolean {
  const wanted = typed.trim().toLowerCase();
  if (wanted === "") return false;
  return !paths.some((path) => path.toLowerCase() === wanted || (path.split("/").pop() ?? path).trim().toLowerCase() === wanted);
}

/** The line under the category field: "\"{typed}\" will be created" and the rule hint, joined by " · ". */
export function categoryHint(typed: string, paths: string[], ruleHint: string | null): string | null {
  const created = isNewCategory(typed, paths) ? `"${typed.trim()}" will be created` : null;
  const parts = [created, ruleHint].filter((part): part is string => part !== null);
  return parts.length > 0 ? parts.join(" · ") : null;
}

/** The options of a suggesting field: all of them, or those containing the typed text ignoring case. */
export function shownOptions(options: string[], typed: string, showAll: boolean): string[] {
  const wanted = typed.trim().toLowerCase();
  return showAll ? options : options.filter((option) => option.toLowerCase().includes(wanted));
}
