import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import SEARCH from "@material-design-icons/svg/round/search.svg?raw";
import {useEffect, useRef} from "react";
import type {LibraryTokenDto_Serialize} from "../../../ipc";
import {Button, Icon, ReasonTooltip} from "../../../shared/components";
import {useStore} from "../../../shared/store";
import {useSession} from "../../../session/session";
import {commandHeld, commandKey} from "../../../shared/platform";
import {SEARCH_FIELD_ATTRIBUTE} from "../interaction/keys";
import {aiMadeFilters, isAiMade, offersPlainWords, turnIntoFilters} from "./plain-words";
import {type FacetFilter, searchFocus} from "../state/view-state";

export type FilterToken = LibraryTokenDto_Serialize;

/**
 * The search field (Kotlin `TokenSearchField`): active filters stand in it as tokens with an ×, a saved view's own
 * conditions as tokens without one. Typing suggests tokens ("+ tag: invoice"), Enter takes the first, Backspace in
 * the empty field removes the last removable token. It takes the focus whenever "/" asks. While the AI is on, three
 * or more words offer "Turn into filters" (⌘/Ctrl+Enter); the filters it makes show as AI-made tokens.
 */
export function TokenSearchField({ query, tokens, suggestions, placeholder, onQuery, onRemove, onTake }: {
  query: string;
  tokens: FilterToken[];
  suggestions: FilterToken[];
  placeholder: string;
  onQuery: (query: string) => void;
  onRemove: (filter: FacetFilter) => void;
  onTake: (filter: FacetFilter) => void;
}) {
  const input = useRef<HTMLInputElement>(null);
  const focusRequests = useStore(searchFocus);
  const first = useRef(focusRequests);
  const session = useSession();
  const aiMade = useStore(aiMadeFilters);
  const plainWords = session.aiEnabled && offersPlainWords(query);
  useEffect(() => {
    if (focusRequests !== first.current) input.current?.focus();
  }, [focusRequests]);
  const onKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter" && commandHeld(event) && plainWords) {
      event.preventDefault();
      void turnIntoFilters(query);
      return;
    }
    const suggestion = suggestions.find((token) => token.filter)?.filter;
    if (event.key === "Enter" && suggestion) {
      event.preventDefault();
      onTake(suggestion);
      return;
    }
    const last = [...tokens].reverse().find((token) => token.filter)?.filter;
    if (event.key === "Backspace" && query === "" && last) {
      event.preventDefault();
      onRemove(last);
    }
  };
  return (
    <div className="lib-search" onClick={() => input.current?.focus()}>
      <Icon svg={SEARCH} />
      {tokens.map((token, index) => (
        <Token key={`${token.key}:${token.value}:${index}`} token={token} aiMade={token.filter !== null && isAiMade(token.filter, aiMade)} onRemove={onRemove} />
      ))}
      <input
        ref={input}
        {...{ [SEARCH_FIELD_ATTRIBUTE]: "" }}
        className="body-medium"
        value={query}
        placeholder={placeholder}
        aria-label="Search the documents"
        onChange={(event) => onQuery(event.target.value)}
        onKeyDown={onKeyDown}
      />
      {plainWords && (
        <Button variant="text" title={`${commandKey}Enter`} onClick={() => void turnIntoFilters(query)}>
          ✦ Turn into filters
        </Button>
      )}
      {suggestions.map((token) => (
        <Button key={`${token.key}:${token.value}`} variant="text" onClick={() => token.filter && onTake(token.filter)}>
          + {token.key}: {token.value}
        </Button>
      ))}
      <span className="lib-key-hint label-small">/</span>
    </div>
  );
}

function Token({ token, aiMade, onRemove }: { token: FilterToken; aiMade: boolean; onRemove: (filter: FacetFilter) => void }) {
  const removable = token.filter;
  return (
    <ReasonTooltip reason={removable ? null : "Part of the saved view"}>
      <span className={[removable ? "lib-token" : "lib-token fixed", aiMade ? "ai" : ""].join(" ").trim()} title={aiMade ? "Made by the AI from your words" : undefined}>
        <span className="lib-token-key label-medium">{token.key}:</span>
        <span className="lib-token-value label-medium">{token.value}</span>
        {removable && (
          <button
            type="button"
            className="lib-token-remove"
            aria-label={`Remove ${token.key}: ${token.value}`}
            onClick={(event) => {
              event.stopPropagation();
              onRemove(removable);
            }}
          >
            <Icon svg={CLOSE} />
          </button>
        )}
      </span>
    </ReasonTooltip>
  );
}
