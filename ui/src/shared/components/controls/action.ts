/** How prominent a button is: one filled main action per area, tonal for frequent ones, text for minor ones. */
export type ButtonStyle = "filled" | "tonal" | "outlined" | "text";

/**
 * A button computed by a screen. A button that cannot be used now names the reason in `disabledReason`, shown
 * as its tooltip. `shortcut` is shown beside it in a menu.
 */
export type ActionButton = {
  label: string;
  icon?: string;
  style?: ButtonStyle;
  disabledReason?: string | null;
  shortcut?: string;
  onClick: () => void;
};

/** Why a button waits: the operation running in its lane. */
export function waitFor(operation: string): string {
  return `Wait until "${operation}" finishes`;
}
