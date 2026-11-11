import ARCHIVE from "@material-design-icons/svg/round/archive.svg?raw";
import {Button, Chip, Dialog, Icon, Switch, TextField} from "../../../shared/components";
import {useStore} from "../../../shared/store";
import {exporting, exportStore, taxYears} from "./export-state";

/** The export being set up: presets, name and layout, then the target folder (Kotlin `ExportDialog`). */
export function ExportDialog() {
  const draft = useStore(exportStore);
  if (!draft) return null;
  const options = draft.options;
  return (
    <Dialog
      width={600}
      style={{ maxWidth: "calc(100vw - 48px)" }}
      onDismiss={exporting.cancel}
      title={
        <div className="column gap-l">
          <span className="lib-dialog-icon"><Icon svg={ARCHIVE} /></span>
          <span className="lib-export-title">Export: {draft.scopeLabel}</span>
        </div>
      }
      buttons={
        <>
          <Button variant="text" onClick={exporting.cancel}>Cancel</Button>
          <Button variant="filled" disabled={!draft.name.trim()} onClick={() => void exporting.run()}>Choose folder and export</Button>
        </>
      }
    >
      <div className="column gap-s">
        <span className="label-large">Presets</span>
        <div className="row wrap gap-s">
          {taxYears().map((year) => (
            <Chip key={year} label={`Tax year ${year}`} onClick={() => exporting.presetTaxYear(year)} />
          ))}
        </div>
        <div className="row gap-s">
          <TextField
            label="Everything for correspondent…"
            value={draft.correspondent}
            onChange={exporting.setCorrespondent}
            labelBackground="var(--surface-high)"
            style={{ flex: 1 }}
          />
          <Button variant="text" disabled={!draft.correspondent.trim()} onClick={exporting.presetCorrespondent}>Use</Button>
        </div>
        <TextField label="Name of the export folder or ZIP file" value={draft.name} onChange={exporting.setName} labelBackground="var(--surface-high)" />
        <OptionRow text="Keep the folder structure" checked={!options.flatten} onChange={(on) => exporting.setOptions({ ...options, flatten: !on })} />
        <OptionRow text="Name files after their title" checked={options.renameToTitle} onChange={(on) => exporting.setOptions({ ...options, renameToTitle: on })} />
        <OptionRow text="Add an index (index.csv and index.html)" checked={options.index} onChange={(on) => exporting.setOptions({ ...options, index: on })} />
        <OptionRow text="One ZIP file instead of a folder" checked={options.zip} onChange={(on) => exporting.setOptions({ ...options, zip: on })} />
      </div>
    </Dialog>
  );
}

function OptionRow({ text, checked, onChange }: { text: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <div className="lib-option-row">
      <span className="body-medium grow">{text}</span>
      <Switch checked={checked} onChange={onChange} label={text} />
    </div>
  );
}
