import MOVE_TO_INBOX from "@material-design-icons/svg/round/move_to_inbox.svg?raw";
import SAVE from "@material-design-icons/svg/round/save.svg?raw";
import { commands, unwrap, type InboxSettingsDto } from "../../../ipc";
import { reportError } from "../../../session/session";
import { ActionButtons, AppCard, CardTitle, Switch, TextField } from "../../../shared/components";
import { useInboxPathProblem } from "../queries";
import { useDraft } from "../use-draft";

/**
 * The inbox: on or off, its folder inside the library (the same in every library folder), and how much the AI does
 * with what arrives, shown only while AI is set up. Edits apply when saved.
 */
export function InboxCard({ inbox, aiEnabled, busyReason }: { inbox: InboxSettingsDto; aiEnabled: boolean; busyReason: string | null }) {
  const [draft, setDraft] = useDraft(inbox, (saved) => saved);
  const problem = useInboxPathProblem(draft.path).data ?? null;
  const change = (part: Partial<InboxSettingsDto>) => setDraft({ ...draft, ...part });
  return (
    <AppCard className="settings-card">
      <CardTitle title="Inbox" icon={MOVE_TO_INBOX} tint="var(--primary)" subtitle="A folder in your library where new documents arrive, on every device" />
      <div className="column gap-m">
        <SwitchRow text="Use an inbox folder" checked={draft.enabled} onChange={(enabled) => change({ enabled })} />
        <TextField
          label="Folder inside the library"
          value={draft.path}
          onChange={(path) => change({ path })}
          error={problem}
          support="Created in every library folder when you save"
          disabled={!draft.enabled}
        />
        {aiEnabled && (
          <>
            <SwitchRow text="Use AI for inbox documents" checked={draft.aiAnalysis} disabled={!draft.enabled} onChange={(aiAnalysis) => change({ aiAnalysis })} />
            <SwitchRow
              text="AI suggests the target folder"
              checked={draft.aiTargetFolder}
              disabled={!draft.enabled || !draft.aiAnalysis}
              onChange={(aiTargetFolder) => change({ aiTargetFolder })}
            />
            <SwitchRow
              text="Analyze automatically when files arrive"
              checked={draft.autoAnalyze}
              disabled={!draft.enabled || !draft.aiAnalysis}
              onChange={(autoAnalyze) => change({ autoAnalyze })}
            />
          </>
        )}
        <ActionButtons
          buttons={[{ label: "Save", icon: SAVE, style: "tonal", disabledReason: problem, onClick: () => void unwrap(commands.saveInbox(draft)).catch(reportError) }]}
          enabled={busyReason === null}
          busyReason={busyReason}
        />
      </div>
    </AppCard>
  );
}

function SwitchRow({ text, checked, disabled, onChange }: { text: string; checked: boolean; disabled?: boolean; onChange: (checked: boolean) => void }) {
  return (
    <div className="row gap-l">
      <span className="body-medium grow">{text}</span>
      <Switch label={text} checked={checked} disabled={disabled} onChange={onChange} />
    </div>
  );
}
