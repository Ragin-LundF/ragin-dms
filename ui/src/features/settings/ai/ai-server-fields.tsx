import type {AiDraftCheckDto_Serialize, AiDraftDto, AiProviderDto} from "../../../ipc";
import {Chip, TextField} from "../../../shared/components";

const DEFAULT_ANTHROPIC_MODEL = "claude-opus-5-5";
const PROVIDERS: { id: AiProviderDto; label: string }[] = [
  { id: "local", label: "Local AI" },
  { id: "anthropic", label: "Anthropic" },
];

/** What each model is for; shown under its field and above its list of models. */
export const CHAT_MODEL_USE =
  "Reads your documents and writes: suggestions, tags, summaries, chat, the answers of Ask and Convert to real PDF.";
export const EMBEDDING_MODEL_USE =
  "Only used by Ask and Related documents: turns text into numbers to find matching passages. Writes no text.";

const digits = (typed: string) => typed.replace(/\D/g, "");

/**
 * How long one answer may take and how much text is sent, then where the AI runs, Local AI or Anthropic, with the
 * fields of that choice. The shared fields come first, so they keep their place on a switch.
 */
export function AiServerFields({ draft, check, onChange }: { draft: AiDraftDto; check: AiDraftCheckDto_Serialize | undefined; onChange: (draft: AiDraftDto) => void }) {
  return (
    <>
      <div className="row gap-m settings-top">
        {/* An error without a supporting text: only the outline and label turn red, as in Compose. */}
        <div className={check?.timeoutValid === false ? "settings-field-error" : undefined}>
          <TextField
            label="Timeout (seconds)"
            value={draft.timeoutSeconds}
            onChange={(typed) => onChange({ ...draft, timeoutSeconds: digits(typed) })}
            style={{ width: 150 }}
          />
        </div>
        <TextField
          label="Text sent to the AI (characters)"
          value={draft.inputCharacters}
          onChange={(typed) => onChange({ ...draft, inputCharacters: digits(typed) })}
          support="1000 to 200000; less for small models"
          error={check?.inputValid === false ? "1000 to 200000; less for small models" : null}
          style={{ width: 260 }}
        />
      </div>
      <div className="column gap-s">
        <span className="label-large muted">Runs on</span>
        <div className="row gap-s" role="radiogroup">
          {PROVIDERS.map((provider) => (
            <Chip key={provider.id} label={provider.label} selected={draft.provider === provider.id} onClick={() => onChange({ ...draft, provider: provider.id })} />
          ))}
        </div>
      </div>
      {draft.provider === "local" ? (
        <>
          <TextField
            label="Server address"
            value={draft.baseUrl}
            onChange={(baseUrl) => onChange({ ...draft, baseUrl })}
            support="LM Studio: http://localhost:1234/v1"
            error={check?.addressValid === false ? "LM Studio: http://localhost:1234/v1" : null}
            style={{ width: 520 }}
          />
          <div className="row gap-m settings-top">
            <TextField
              label="Chat model"
              value={draft.model}
              placeholder="First loaded model"
              onChange={(model) => onChange({ ...draft, model })}
              support={CHAT_MODEL_USE}
              style={{ width: 360 }}
            />
            <TextField
              label="Embedding model"
              value={draft.embeddingModel}
              placeholder="First embedding model of the server"
              onChange={(embeddingModel) => onChange({ ...draft, embeddingModel })}
              support={EMBEDDING_MODEL_USE}
              style={{ width: 360 }}
            />
          </div>
        </>
      ) : (
        <>
          <TextField
            label="Claude model"
            value={draft.anthropicModel}
            placeholder={DEFAULT_ANTHROPIC_MODEL}
            onChange={(anthropicModel) => onChange({ ...draft, anthropicModel })}
            style={{ width: 360 }}
          />
          <span className="body-small muted settings-note">
            The API key is read from ANTHROPIC_API_KEY when the app starts. Ask still uses the embedding model of Local AI.
          </span>
        </>
      )}
    </>
  );
}
