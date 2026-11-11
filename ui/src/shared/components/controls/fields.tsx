import type {CSSProperties, ReactNode} from "react";
import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import SEARCH from "@material-design-icons/svg/round/search.svg?raw";
import {IconButton} from "./buttons";
import {Icon} from "./icon";

/**
 * An outlined text field with its label in the border. `labelBackground` must match the surface the field sits
 * on, so the border line stops behind the label.
 */
export function TextField({ label, value, onChange, placeholder, error, support, disabled, autoFocus, multiline, rows, onEnter, onEscape, leading, trailing, dense, labelBackground, style, type, inputRef, onBlur }: {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  error?: string | null;
  support?: string;
  disabled?: boolean;
  autoFocus?: boolean;
  multiline?: boolean;
  rows?: number;
  onEnter?: () => void;
  onEscape?: () => void;
  leading?: ReactNode;
  trailing?: ReactNode;
  dense?: boolean;
  labelBackground?: string;
  style?: CSSProperties;
  type?: string;
  inputRef?: React.Ref<HTMLInputElement & HTMLTextAreaElement>;
  onBlur?: () => void;
}) {
  const classes = ["field", error ? "error" : "", disabled ? "disabled" : "", dense ? "dense" : ""].filter(Boolean).join(" ");
  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Enter" && !(multiline && event.shiftKey) && onEnter) {
      event.preventDefault();
      onEnter();
    }
    if (event.key === "Escape" && onEscape) {
      event.stopPropagation();
      onEscape();
    }
  };
  const common = {
    value,
    placeholder,
    disabled,
    autoFocus,
    onKeyDown,
    onBlur,
    "aria-label": label ?? placeholder,
    onChange: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => onChange(event.target.value),
  };
  return (
    <div style={style}>
      <div className={classes} style={labelBackground ? ({ "--field-label-bg": labelBackground } as CSSProperties) : undefined}>
        {label && <label>{label}</label>}
        {leading}
        {multiline ? <textarea ref={inputRef} rows={rows ?? 3} {...common} /> : <input ref={inputRef} type={type ?? "text"} {...common} />}
        {trailing}
      </div>
      {(error || support) && <div className={error ? "field-support error" : "field-support"}>{error ?? support}</div>}
    </div>
  );
}

/** A rounded search field with a clear button; `onSearch` runs on Enter. */
export function SearchField({ query, onQueryChange, placeholder, onSearch, style, inputRef }: {
  query: string;
  onQueryChange: (query: string) => void;
  placeholder: string;
  onSearch?: () => void;
  style?: CSSProperties;
  inputRef?: React.Ref<HTMLInputElement & HTMLTextAreaElement>;
}) {
  return (
    <div className="field search-field" style={style}>
      <span className="field-icon"><Icon svg={SEARCH} /></span>
      <input ref={inputRef} value={query} placeholder={placeholder} aria-label={placeholder} onChange={(event) => onQueryChange(event.target.value)} onKeyDown={(event) => event.key === "Enter" && onSearch?.()} />
      {query && <IconButton icon={CLOSE} label="Clear" onClick={() => onQueryChange("")} />}
    </div>
  );
}

/** A Material 3 switch. */
export function Switch({ checked, onChange, disabled, label }: { checked: boolean; onChange: (checked: boolean) => void; disabled?: boolean; label: string }) {
  return <button type="button" role="switch" aria-checked={checked} aria-label={label} disabled={disabled} className={checked ? "switch on" : "switch"} onClick={() => onChange(!checked)} />;
}

/** A filter chip; `selected` fills it. */
export function Chip({ label, selected, onClick, icon, disabled, title }: { label: ReactNode; selected?: boolean; onClick?: () => void; icon?: string; disabled?: boolean; title?: string }) {
  return (
    <button type="button" title={title} aria-pressed={selected} disabled={disabled} className={["chip", selected ? "selected" : "", icon ? "has-icon" : ""].filter(Boolean).join(" ")} onClick={onClick}>
      {icon && <Icon svg={icon} />}
      {label}
    </button>
  );
}
