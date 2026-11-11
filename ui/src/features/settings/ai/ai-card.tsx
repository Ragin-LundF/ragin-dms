import {useState} from "react";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import NETWORK_CHECK from "@material-design-icons/svg/round/network_check.svg?raw";
import SAVE from "@material-design-icons/svg/round/save.svg?raw";
import {type AiDraftDto, type AiProviderDto, type AiSettingsDto_Serialize, commands, unwrap} from "../../../ipc";
import {reportError} from "../../../session/session";
import {type ActionButton, ActionButtons, AppCard, CardTitle} from "../../../shared/components";
import {useStore} from "../../../shared/store";
import {useAiDraftCheck} from "../queries";
import {useDraft} from "../use-draft";
import {AiConnectionLine, AiModelChips, connectionStore, isEmbeddingModel} from "./ai-connection";
import {AiServerFields, CHAT_MODEL_USE, EMBEDDING_MODEL_USE} from "./ai-server-fields";
import {SettingSwitch} from "./setting-switch";

function draftOf(settings: AiSettingsDto_Serialize): AiDraftDto {
  return {
    enabled: settings.enabled,
    baseUrl: settings.baseUrl,
    model: settings.model ?? "",
    timeoutSeconds: String(settings.timeoutSeconds),
    preferVisionForPdfs: settings.preferVisionForPdfs,
    acceptSuggestedFileNames: settings.acceptSuggestedFileNames,
    embeddingModel: settings.embeddingModel ?? "",
    inputCharacters: String(settings.inputCharacters),
    provider: settings.provider,
    anthropicModel: settings.anthropicModel,
  };
}

/** Asks the server for its models with the typed settings, without saving them. */
function testConnection(settings: AiSettingsDto_Serialize) {
  connectionStore.set({ checking: true, result: null });
  unwrap(commands.testAiConnection(settings)).then(
    (result) => connectionStore.set({ checking: false, result }),
    (error: unknown) => {
      connectionStore.set({ checking: false, result: null });
      reportError(error);
    },
  );
}

/** Where the AI runs, Local AI or Anthropic, its model, and a test that lists the provider's models. */
export function AiSettingsCard({ settings, busyReason }: { settings: AiSettingsDto_Serialize; busyReason: string | null }) {
  const [draft, setDraft] = useDraft(settings, draftOf);
  const check = useAiDraftCheck(draft, settings).data;
  const connection = useStore(connectionStore);
  // The listed models belong to the provider that was tested; another provider's names would not work.
  const [testedProvider, setTestedProvider] = useState<AiProviderDto | null>(null);
  const typed = check?.settings ?? null;
  const models = connection.result?.models ?? [];
  const buttons: ActionButton[] = [];
  if (draft.enabled) {
    buttons.push({
      label: "Test connection",
      icon: NETWORK_CHECK,
      disabledReason: check ? check.problem : "",
      onClick: () => {
        if (!typed) return;
        setTestedProvider(typed.provider);
        testConnection(typed);
      },
    });
  }
  buttons.push({
    label: "Save",
    icon: SAVE,
    style: "tonal",
    disabledReason: check ? check.saveDisabledReason : "",
    onClick: () => typed && void unwrap(commands.saveAiSettings(typed)).catch(reportError),
  });
  return (
    <AppCard className="settings-card">
      <CardTitle title="AI" icon={AUTO_AWESOME} tint="var(--primary)" subtitle="Local server or Anthropic. It only suggests; you decide" />
      <div className="column gap-m">
        <SettingSwitch
          title="Use AI"
          text="Shows the AI features: suggestions, tags, summaries, questions and converting scans to real PDFs. When off, the app shows none of them and sends nothing anywhere."
          checked={draft.enabled}
          onChange={(enabled) => setDraft({ ...draft, enabled })}
        />
        {draft.enabled && (
          <>
            {/* The general switches come first, so they keep their place when the provider fields change. */}
            <SettingSwitch
              title="Read PDFs as images"
              text="Sends rendered pages even when a PDF has text. Helps with poor scanner text; needs a vision model and is slower."
              checked={draft.preferVisionForPdfs}
              onChange={(preferVisionForPdfs) => setDraft({ ...draft, preferVisionForPdfs })}
            />
            <SettingSwitch
              title="Rename to the suggested file name"
              text="Checks the suggested file name in a review, so approving renames the file. When off, the name is still suggested and you check it if it is better."
              checked={draft.acceptSuggestedFileNames}
              onChange={(acceptSuggestedFileNames) => setDraft({ ...draft, acceptSuggestedFileNames })}
            />
            <AiServerFields draft={draft} check={check} onChange={setDraft} />
            {models.length > 0 && testedProvider === draft.provider && draft.provider === "anthropic" && (
              <AiModelChips
                models={models}
                selected={draft.anthropicModel}
                firstLabel="Default model"
                onSelect={(anthropicModel) => setDraft({ ...draft, anthropicModel })}
              />
            )}
            {models.length > 0 && testedProvider === draft.provider && draft.provider === "local" && (
              <LocalModelChoice models={models} draft={draft} onChange={setDraft} />
            )}
            <AiConnectionLine checking={connection.checking} result={connection.result} />
          </>
        )}
        <ActionButtons buttons={buttons} enabled={busyReason === null} busyReason={busyReason} />
      </div>
    </AppCard>
  );
}

/**
 * The local server's models in two lists, so each can be picked by a click: chat models for everything that writes,
 * embedding models for Ask. A server without a model that looks like an embedding model offers all models there.
 */
function LocalModelChoice({ models, draft, onChange }: { models: string[]; draft: AiDraftDto; onChange: (draft: AiDraftDto) => void }) {
  const embedding = models.filter(isEmbeddingModel);
  const chat = models.filter((model) => !isEmbeddingModel(model));
  return (
    <div className="column gap-m">
      <div className="column gap-s">
        <span className="label-large">Chat model</span>
        <span className="body-small muted">{CHAT_MODEL_USE}</span>
        <AiModelChips models={chat} selected={draft.model} firstLabel="First loaded model" onSelect={(model) => onChange({ ...draft, model })} />
      </div>
      <div className="column gap-s">
        <span className="label-large">Embedding model</span>
        <span className="body-small muted">
          {EMBEDDING_MODEL_USE}
          {embedding.length === 0 && " The server lists no embedding model by name, so all models are shown; load one such as nomic-embed-text in LM Studio."}
        </span>
        <AiModelChips
          models={embedding.length > 0 ? embedding : models}
          selected={draft.embeddingModel}
          firstLabel="First embedding model"
          onSelect={(embeddingModel) => onChange({ ...draft, embeddingModel })}
        />
      </div>
    </div>
  );
}
