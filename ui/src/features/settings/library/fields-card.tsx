import { useState } from "react";
import TEXT_FIELDS from "@material-design-icons/svg/round/text_fields.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import ADD from "@material-design-icons/svg/round/add.svg?raw";
import PLAYLIST_ADD from "@material-design-icons/svg/round/playlist_add.svg?raw";
import ARROW_DROP_DOWN from "@material-design-icons/svg/round/arrow_drop_down.svg?raw";
import type { FieldRoleDto, FieldTypeDto, LibrarySettingsDto } from "../../../ipc";
import { ActionButtons, AppCard, Button, CardTitle, Icon, IconButton, Menu, MenuItem, Switch, TextField, useMenuAnchor } from "../../../shared/components";
import { useDraft } from "../use-draft";
import { changeLibrarySettings } from "./change";

const NO_DEADLINE = "No deadline";

/**
 * The library's custom fields: each with its type and, for dates, the deadline it stands for; a new one by label,
 * type and role; and the currency money values take when they name none.
 */
export function FieldsCard({ settings, busyReason }: { settings: LibrarySettingsDto; busyReason: string | null }) {
  const enabled = busyReason === null;
  return (
    <AppCard className="settings-card">
      <CardTitle title="Custom fields" icon={TEXT_FIELDS} tint="var(--secondary)" subtitle="Values such as amounts and deadlines, on every document, for search and the dashboard" />
      <div className="column gap-s">
        {settings.fields.map((field) => (
          <div key={field.key} className="row">
            <span className="body-medium grow settings-keep-spaces">{field.text}</span>
            <IconButton icon={DELETE} label={`Remove ${field.label}`} disabled={!enabled} onClick={() => changeLibrarySettings({ kind: "removeField", key: field.key })} />
          </div>
        ))}
        <NewField settings={settings} busyReason={busyReason} />
        <CurrencyRow current={settings.defaultCurrency} busyReason={busyReason} />
        <div className="row">
          <span className="body-medium grow">Also write approved title, correspondent, category, tags and date into PDF files</span>
          <Switch
            label="Also write approved title, correspondent, category, tags and date into PDF files"
            checked={settings.embedMetadata}
            disabled={!enabled}
            onChange={(on) => changeLibrarySettings({ kind: "setEmbedMetadata", enabled: on })}
          />
        </div>
      </div>
    </AppCard>
  );
}

function NewField({ settings, busyReason }: { settings: LibrarySettingsDto; busyReason: string | null }) {
  const [label, setLabel] = useState("");
  const [type, setType] = useState<FieldTypeDto>("text");
  const [role, setRole] = useState<FieldRoleDto | null>(null);
  const typeLabel = settings.fieldTypes.find((choice) => choice.value === type)?.label ?? type;
  const roleLabel = settings.fieldRoles.find((choice) => choice.value === role)?.label ?? NO_DEADLINE;
  return (
    <>
      <div className="row gap-s">
        <TextField label="New field" value={label} onChange={setLabel} style={{ flex: 1 }} />
        <Choice label={typeLabel} options={settings.fieldTypes.map((choice) => ({ label: choice.label, pick: () => setType(choice.value) }))} />
        {type === "date" && (
          <Choice
            label={roleLabel}
            options={[{ label: NO_DEADLINE, pick: () => setRole(null) }, ...settings.fieldRoles.map((choice) => ({ label: choice.label, pick: () => setRole(choice.value) }))]}
          />
        )}
      </div>
      <ActionButtons
        buttons={[
          {
            label: "Add field",
            icon: ADD,
            disabledReason: label.trim() === "" ? "Name the field" : null,
            onClick: () => {
              changeLibrarySettings({ kind: "addField", label, fieldType: type, role });
              setLabel("");
            },
          },
          { label: "Add standard fields", icon: PLAYLIST_ADD, onClick: () => changeLibrarySettings({ kind: "addStandardFields" }) },
        ]}
        enabled={busyReason === null}
        busyReason={busyReason}
      />
    </>
  );
}

function CurrencyRow({ current, busyReason }: { current: string; busyReason: string | null }) {
  const [typed, setTyped] = useDraft(current, (saved) => saved);
  return (
    <div className="row gap-s">
      <TextField label="Default currency, such as EUR" value={typed} onChange={setTyped} style={{ flex: 1 }} />
      <ActionButtons
        buttons={[
          {
            label: "Save currency",
            disabledReason: typed.trim() === current ? "Nothing changed" : null,
            onClick: () => changeLibrarySettings({ kind: "saveCurrency", typed }),
          },
        ]}
        enabled={busyReason === null}
        busyReason={busyReason}
      />
    </div>
  );
}

/** An outlined button that opens its options as a dropdown. */
function Choice({ label, options }: { label: string; options: { label: string; pick: () => void }[] }) {
  const menu = useMenuAnchor();
  return (
    <>
      <Button variant="outlined" className="settings-choice" onClick={menu.open}>
        <span className="nowrap">{label}</span>
        <Icon svg={ARROW_DROP_DOWN} />
      </Button>
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {options.map((option) => (
            <MenuItem
              key={option.label}
              label={option.label}
              onClick={() => {
                option.pick();
                menu.close();
              }}
            />
          ))}
        </Menu>
      )}
    </>
  );
}
