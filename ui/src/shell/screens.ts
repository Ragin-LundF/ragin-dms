import DASHBOARD from "@material-design-icons/svg/round/dashboard.svg?raw";
import INBOX from "@material-design-icons/svg/round/move_to_inbox.svg?raw";
import TASKS from "@material-design-icons/svg/round/checklist.svg?raw";
import LIBRARY from "@material-design-icons/svg/round/folder.svg?raw";
import ASK from "@material-design-icons/svg/round/question_answer.svg?raw";
import BILLS from "@material-design-icons/svg/round/payments.svg?raw";
import CONTRACTS from "@material-design-icons/svg/round/autorenew.svg?raw";
import TAX from "@material-design-icons/svg/round/receipt_long.svg?raw";
import CASES from "@material-design-icons/svg/round/forum.svg?raw";
import EMERGENCY from "@material-design-icons/svg/round/emergency.svg?raw";
import RESTRUCTURE from "@material-design-icons/svg/round/account_tree.svg?raw";
import SETTINGS from "@material-design-icons/svg/round/settings.svg?raw";
import {createStore, useStore} from "../shared/store";

export type Screen = "dashboard" | "inbox" | "tasks" | "library" | "ask" | "bills" | "contracts" | "tax" | "cases" | "emergency" | "restructure" | "settings";

/** The groups of the navigation, each under a small label. */
export const NAV_GROUPS = [
  { id: "home", label: "Home" },
  { id: "documents", label: "Documents" },
  { id: "organize", label: "Organize" },
  { id: "maintain", label: "Maintain" },
] as const;

/** The main screens, in the order of the navigation, with their group; null is pinned to the bottom. */
export const SCREENS: { id: Screen; label: string; icon: string; group: (typeof NAV_GROUPS)[number]["id"] | null }[] = [
  { id: "dashboard", label: "Dashboard", icon: DASHBOARD, group: "home" },
  { id: "inbox", label: "Inbox", icon: INBOX, group: "home" },
  { id: "tasks", label: "Open tasks", icon: TASKS, group: "home" },
  { id: "library", label: "Library", icon: LIBRARY, group: "documents" },
  { id: "ask", label: "Ask", icon: ASK, group: "documents" },
  { id: "bills", label: "To pay", icon: BILLS, group: "organize" },
  { id: "contracts", label: "Contracts", icon: CONTRACTS, group: "organize" },
  { id: "tax", label: "Tax years", icon: TAX, group: "organize" },
  { id: "cases", label: "Cases", icon: CASES, group: "organize" },
  { id: "emergency", label: "Emergency folder", icon: EMERGENCY, group: "organize" },
  { id: "restructure", label: "Restructure", icon: RESTRUCTURE, group: "maintain" },
  { id: "settings", label: "Settings", icon: SETTINGS, group: null },
];

export function screenLabel(screen: Screen): string {
  return SCREENS.find((entry) => entry.id === screen)?.label ?? screen;
}

/**
 * Where the window is: the screen, a request for it (such as "open the library at this folder"), the log panel
 * and the navigation's fold. `request` is consumed by the screen it names, see [`useScreenRequest`].
 */
export type NavigationState = {
  screen: Screen;
  request: { screen: Screen; payload: unknown; id: number } | null;
  logVisible: boolean;
  collapsed: boolean;
  paletteOpen: boolean;
};

export const navigation = createStore<NavigationState>({ screen: "dashboard", request: null, logVisible: false, collapsed: false, paletteOpen: false });

let requestId = 0;

/** Shows `screen`; with `payload`, the screen acts on it once (opens a folder, asks a question, …). */
export function goTo(screen: Screen, payload?: unknown) {
  navigation.set((state) => ({
    ...state,
    screen,
    request: payload === undefined ? state.request : { screen, payload, id: ++requestId },
  }));
}

export function useNavigation(): NavigationState {
  return useStore(navigation);
}

export function setLogVisible(visible: boolean) {
  navigation.set((state) => ({ ...state, logVisible: visible }));
}

export function setPaletteOpen(open: boolean) {
  navigation.set((state) => ({ ...state, paletteOpen: open }));
}

/**
 * The request sent to `screen` with [`goTo`], once: `onRequest` runs for every new one, and it is cleared.
 * Screens call it from an effect: `useScreenRequest("library", (payload) => …)`.
 */
export function takeRequest<T>(screen: Screen): T | undefined {
  const request = navigation.get().request;
  if (!request || request.screen !== screen) return undefined;
  navigation.set((state) => ({ ...state, request: null }));
  return request.payload as T;
}
