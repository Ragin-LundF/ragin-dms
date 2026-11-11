import ACCOUNT_TREE from "@material-design-icons/svg/round/account_tree.svg?raw";
import ARROW_DROP_DOWN from "@material-design-icons/svg/round/arrow_drop_down.svg?raw";
import PREVIEW from "@material-design-icons/svg/round/preview.svg?raw";
import SAVE from "@material-design-icons/svg/round/save.svg?raw";
import type {RestructureStateDto_Serialize, ScopeOptionDto} from "../../ipc";
import {
    ActionButtons,
    AppCard,
    CardTitle,
    Chip,
    Icon,
    Menu,
    MenuItem,
    Switch,
    TextField,
    useMenuAnchor
} from "../../shared/components";
import {type RestructureDrafts, updateDrafts, withPlaceholder} from "./drafts";

/**
 * What to restructure and how: the scope, the path template with a chip per placeholder, the file naming, and the
 * buttons to preview the plan and to keep the template as the library's default.
 */
export function TemplateCard({ state, drafts, scopeIndex, busyReason, onPreview, onSave }: {
  state: RestructureStateDto_Serialize;
  drafts: RestructureDrafts;
  scopeIndex: number;
  busyReason: string | null;
  onPreview: () => void;
  onSave: () => void;
}) {
  return (
    <AppCard className="restructure-card">
      <CardTitle title="Template" icon={ACCOUNT_TREE} tint="var(--primary)" subtitle="Folders separated by /, values in braces, a fallback after |, such as {year|Undated}" />
      <ScopePicker scopes={state.scopes} index={scopeIndex} onSelect={(option) => updateDrafts({ scope: option.scope, planned: false })} />
      <TextField
        label="Path template"
        value={drafts.template}
        error={drafts.problem}
        labelBackground="var(--surface)"
        onChange={(template) => updateDrafts({ template, problem: null, planned: false })}
      />
      <div className="row wrap gap-s">
        {state.placeholders.map((token) => (
          <Chip key={token} label={`{${token}}`} onClick={() => updateDrafts({ template: withPlaceholder(drafts.template, token), planned: false })} />
        ))}
      </div>
      <div className="row gap-m">
        <Switch
          label={'Rename to "date title" where a document has both'}
          checked={drafts.naming === "dateAndTitle"}
          onChange={(renamed) => updateDrafts({ naming: renamed ? "dateAndTitle" : "keep", planned: false })}
        />
        <span className="body-medium">Rename to "date title" where a document has both</span>
      </div>
      <ActionButtons
        enabled={busyReason === null}
        busyReason={busyReason}
        buttons={[
          { label: "Preview", icon: PREVIEW, style: "tonal", onClick: onPreview },
          {
            label: "Save as default",
            icon: SAVE,
            disabledReason: state.savedTemplate === drafts.template.trim() ? "This is the saved template" : null,
            onClick: onSave,
          },
        ]}
      />
    </AppCard>
  );
}

function ScopePicker({ scopes, index, onSelect }: { scopes: ScopeOptionDto[]; index: number; onSelect: (option: ScopeOptionDto) => void }) {
  const menu = useMenuAnchor();
  return (
    <div>
      <button type="button" className="btn btn-outlined restructure-scope" onClick={menu.open}>
        <span className="ellipsis">Scope: {scopes[index]?.label ?? ""}</span>
        <Icon svg={ARROW_DROP_DOWN} />
      </button>
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {scopes.map((option) => (
            <MenuItem
              key={option.label}
              label={option.label}
              onClick={() => {
                menu.close();
                onSelect(option);
              }}
            />
          ))}
        </Menu>
      )}
    </div>
  );
}
