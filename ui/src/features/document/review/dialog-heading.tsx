import { Icon } from "../../../shared/components";

/** The title of a Material 3 dialog with an icon: the icon centred above the centred title. */
export function DialogHeading({ icon, title }: { icon: string; title: string }) {
  return (
    <span className="dialog-heading">
      <Icon svg={icon} size={24} />
      <span>{title}</span>
    </span>
  );
}
