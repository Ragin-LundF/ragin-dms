import ERROR_OUTLINE from "@material-design-icons/svg/round/error_outline.svg?raw";
import CHECK_CIRCLE from "@material-design-icons/svg/round/check_circle.svg?raw";
import type {AiConnectionDto} from "../../../ipc";
import {Chip, Icon, Spinner} from "../../../shared/components";
import {createStore} from "../../../shared/store";

/** The last connection test: running, or what it found. Kept while the app runs, like the Kotlin view model's. */
export const connectionStore = createStore<{ checking: boolean; result: AiConnectionDto | null }>({ checking: false, result: null });

/** What the last connection test found, with an icon so the result does not rely on colour. */
export function AiConnectionLine({ checking, result }: { checking: boolean; result: AiConnectionDto | null }) {
  if (!checking && !result) return null;
  const isError = !checking && result?.isError;
  return (
    <div className="row gap-s">
      {checking ? <Spinner small /> : <Icon svg={isError ? ERROR_OUTLINE : CHECK_CIRCLE} className={isError ? "error-text" : "primary-text"} />}
      <span className={isError ? "body-small error-text" : "body-small muted"}>{checking ? "Asking the server for its models…" : result?.message}</span>
    </div>
  );
}

/**
 * The models the server offered in the last test; picking one fills the model field. The first chip, `firstLabel`,
 * leaves the field blank: the server's first loaded model, or Anthropic's default one.
 */
export function AiModelChips({ models, selected, firstLabel, onSelect }: { models: string[]; selected: string; firstLabel: string; onSelect: (model: string) => void }) {
  return (
    <div className="row wrap gap-s">
      <Chip label={firstLabel} selected={selected.trim() === ""} onClick={() => onSelect("")} />
      {models.map((model) => <Chip key={model} label={<span className="nowrap">{model}</span>} selected={selected === model} onClick={() => onSelect(model)} />)}
    </div>
  );
}

/** Name parts of models that cannot chat, as the Rust client's `NOT_FOR_CHAT` skips them. */
const NOT_FOR_CHAT = ["embed", "rerank", "e5-", "bge-", "gte-"];

/** `true` for models that only make embeddings (or rerank), judged by name like the Rust client does. */
export function isEmbeddingModel(model: string): boolean {
  const name = model.toLowerCase();
  return NOT_FOR_CHAT.some((part) => name.includes(part));
}
