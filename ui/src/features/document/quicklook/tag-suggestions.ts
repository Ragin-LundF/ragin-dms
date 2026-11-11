const TAG_SUGGESTIONS = 6;

/**
 * Library tags that start with the typed text ignoring case, without the document's own tags, in library order; at
 * most six, none for empty input (Kotlin `QuickFactList.tagSuggestions`).
 */
export function tagSuggestions(typed: string, documentTags: string[], libraryTags: string[]): string[] {
  const prefix = typed.trim().toLowerCase();
  if (prefix === "") return [];
  const own = new Set(documentTags.map((tag) => tag.toLowerCase()));
  return libraryTags.filter((tag) => !own.has(tag.toLowerCase()) && tag.toLowerCase().startsWith(prefix)).slice(0, TAG_SUGGESTIONS);
}
