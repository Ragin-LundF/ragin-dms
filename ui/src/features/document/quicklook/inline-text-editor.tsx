import {useEffect, useRef, useState} from "react";

/**
 * A value edited in place (Kotlin `InlineTextEditor`): it starts focused with `initial` selected, Enter saves, Esc
 * cancels, and an unchanged value cancels too. `suggestionsFor` offers values for what is typed; a click on one
 * saves it. Losing the focus keeps the edit open, so a refused value can still be fixed.
 */
export function InlineTextEditor({ initial, description, onSave, onCancel, suggestionsFor }: {
  initial: string;
  description: string;
  onSave: (typed: string) => void;
  onCancel: () => void;
  suggestionsFor?: (typed: string) => string[];
}) {
  const [text, setText] = useState(initial);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);
  const commit = () => (text.trim() === initial.trim() ? onCancel() : onSave(text));
  const suggestions = suggestionsFor?.(text) ?? [];
  return (
    <div className="column gap-xs">
      <input
        ref={input}
        className="inline-editor body-small"
        aria-label={description}
        value={text}
        onChange={(event) => setText(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            commit();
          } else if (event.key === "Escape") {
            event.preventDefault();
            event.stopPropagation();
            onCancel();
          }
          event.stopPropagation();
        }}
      />
      {suggestions.length > 0 && (
        <div className="row wrap gap-xs">
          {suggestions.map((suggestion) => (
            <button key={suggestion} type="button" className="suggestion-chip" onClick={() => onSave(suggestion)}>
              {suggestion}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
