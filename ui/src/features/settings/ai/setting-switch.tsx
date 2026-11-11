import {Switch} from "../../../shared/components";

/** A setting that is on or off: its title, a text that explains it, and the switch. */
export function SettingSwitch({ title, text, checked, onChange }: { title: string; text: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <div className="row gap-l">
      <div className="column grow">
        <span className="title-small">{title}</span>
        <span className="body-small muted">{text}</span>
      </div>
      <Switch label={title} checked={checked} onChange={onChange} />
    </div>
  );
}
