import {commands, unwrap} from "../../../ipc";
import {report, reportError} from "../../../session/session";
import {createStore} from "../../../shared/store";
import {type FacetFilter, sameFilter, viewStore} from "../state/view-state";

/** The filters "Turn into filters" made, so their chips show as AI-made; they behave like any other filter. */
export const aiMadeFilters = createStore<FacetFilter[]>([]);

/** The search field offers "Turn into filters" from this many words on. */
const MIN_WORDS = 3;

/** Facets that may hold several filters at once; every other facet holds one. */
const STACKING: FacetFilter["kind"][] = ["tag", "field"];

export function offersPlainWords(query: string): boolean {
  return query.trim().split(/\s+/).filter(Boolean).length >= MIN_WORDS;
}

/** `filters` with `added`: an added filter takes the place of its facet's filter, except tags and fields, which add up. */
export function withAiFilters(filters: FacetFilter[], added: FacetFilter[]): FacetFilter[] {
  const replaced = new Set(added.map((filter) => filter.kind).filter((kind) => !STACKING.includes(kind)));
  const kept = filters.filter((filter) => !replaced.has(filter.kind) && !added.some((other) => sameFilter(filter, other)));
  return [...kept, ...added];
}

export function isAiMade(filter: FacetFilter, made: FacetFilter[]): boolean {
  return made.some((other) => sameFilter(filter, other));
}

/** Asks the AI to turn `sentence` into filters and applies them; the words it could not place stay in the field. */
export async function turnIntoFilters(sentence: string) {
  try {
    const result = await unwrap(commands.plainSearchTranslate(sentence));
    viewStore.set((state) => ({ ...state, filters: withAiFilters(state.filters, result.filters), query: result.text }));
    aiMadeFilters.set((made) => [...made, ...result.filters]);
    if (result.notUnderstood.length > 0) {
      report({ text: `Not understood: ${result.notUnderstood.join(", ")}`, isError: false });
    } else if (result.filters.length === 0 && result.text === "") {
      report({ text: "No filter found in these words", isError: false });
    }
  } catch (error) {
    reportError(error);
  }
}
