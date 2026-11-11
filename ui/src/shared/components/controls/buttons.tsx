import FOLD from "@material-design-icons/svg/round/indeterminate_check_box.svg?raw";
import UNFOLD from "@material-design-icons/svg/round/add_box.svg?raw";
import type {ButtonHTMLAttributes, ReactNode} from "react";
import {Icon} from "./icon";
import type {ActionButton, ButtonStyle} from "./action";
import {ReasonTooltip} from "./tooltip";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonStyle;
  icon?: string;
  danger?: boolean;
  small?: boolean;
};

/** One Material 3 button with its icon in front of the label. */
export function Button({ variant = "outlined", icon, danger, small, className, children, ...rest }: ButtonProps) {
  const classes = ["btn", `btn-${variant}`, icon ? "has-icon" : "", danger ? "btn-error" : "", small ? "btn-small" : "", className ?? ""];
  return (
    <button type="button" className={classes.filter(Boolean).join(" ")} {...rest}>
      {icon && <Icon svg={icon} />}
      {children}
    </button>
  );
}

type IconButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & { icon: string; label: string; small?: boolean; selected?: boolean; tooltip?: boolean };

/** An icon-only button; `label` names it for screen readers and, with `tooltip`, on hover. */
export function IconButton({ icon, label, small, selected, tooltip, className, ...rest }: IconButtonProps) {
  const button = (
    <button
      type="button"
      aria-label={label}
      className={["icon-btn", small ? "small" : "", selected ? "selected" : "", className ?? ""].filter(Boolean).join(" ")}
      {...rest}
    >
      <Icon svg={icon} />
    </button>
  );
  return tooltip ? <ReasonTooltip reason={label}>{button}</ReasonTooltip> : button;
}

/** A computed button; a disabled one explains in its tooltip why it cannot be used. */
export function StyledButton({ button, enabled = true }: { button: ActionButton; enabled?: boolean }) {
  return (
    <Button variant={button.style ?? "outlined"} icon={button.icon} disabled={!enabled} onClick={button.onClick}>
      {button.label}
    </Button>
  );
}

/** A row of buttons that wraps when narrow; a disabled one explains in its tooltip why it cannot be used. */
export function ActionButtons({ buttons, enabled = true, busyReason }: { buttons: ActionButton[]; enabled?: boolean; busyReason?: string | null }) {
  return (
    <div className="row wrap gap-s">
      {buttons.map((button) => {
        const reason = button.disabledReason ?? (enabled ? null : busyReason ?? null);
        return (
          <ReasonTooltip key={button.label} reason={reason}>
            <StyledButton button={button} enabled={reason === null && enabled} />
          </ReasonTooltip>
        );
      })}
    </div>
  );
}

/** The boxed + or − that unfolds or folds an entry; with `expanded` null the space stays empty so names line up. */
export function FoldToggle({ expanded, onToggle }: { expanded: boolean | null; onToggle: () => void }): ReactNode {
  if (expanded === null) return <span className="fold-toggle" aria-hidden />;
  return (
    <button type="button" className="fold-toggle" aria-label={expanded ? "Fold" : "Unfold"} onClick={(event) => { event.stopPropagation(); onToggle(); }}>
      <Icon svg={expanded ? FOLD : UNFOLD} />
    </button>
  );
}

