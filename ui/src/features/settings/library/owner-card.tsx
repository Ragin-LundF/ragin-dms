import BADGE from "@material-design-icons/svg/round/badge.svg?raw";
import type {SettingsOwnerDto} from "../../../ipc";
import {ActionButtons, AppCard, CardTitle, TextField} from "../../../shared/components";
import {useDraft} from "../use-draft";
import {changeLibrarySettings} from "./change";

/** The owner's name, address and contact for the sender block of letters; stored with the library, on every device. */
export function OwnerCard({ owner, busyReason }: { owner: SettingsOwnerDto; busyReason: string | null }) {
  const [draft, setDraft] = useDraft(owner, (saved) => saved);
  const set = (field: keyof SettingsOwnerDto) => (value: string) => setDraft({ ...draft, [field]: value });
  const unchanged = JSON.stringify(draft) === JSON.stringify(owner);
  return (
    <AppCard className="settings-card">
      <CardTitle title="Your details (for letters)" icon={BADGE} tint="var(--primary)" subtitle="The sender of the letters the app drafts for you; shared by every device of this library" />
      <div className="column gap-s">
        <TextField label="Name" value={draft.name} onChange={set("name")} />
        <TextField label="Address" value={draft.address} onChange={set("address")} multiline rows={3} support="One line per address line" />
        <div className="row gap-s">
          <TextField label="Email" value={draft.email} onChange={set("email")} style={{ flex: 1 }} />
          <TextField label="Phone" value={draft.phone} onChange={set("phone")} style={{ flex: 1 }} />
        </div>
        <ActionButtons
          buttons={[{ label: "Save details", disabledReason: unchanged ? "Nothing changed" : null, onClick: () => changeLibrarySettings({ kind: "saveOwner", owner: draft }) }]}
          enabled={busyReason === null}
          busyReason={busyReason}
        />
      </div>
    </AppCard>
  );
}
