import {useEffect} from "react";
import {useQuery} from "@tanstack/react-query";
import {commands} from "../ipc";
import {AskScreen} from "../features/ask";
import {BillsDialogs, BillsScreen} from "../features/bills";
import {CaseDialogs, CasesScreen} from "../features/cases";
import {ContractsScreen} from "../features/contracts";
import {DashboardScreen, TasksScreen} from "../features/dashboard";
import {DocumentDialogs} from "../features/document";
import {EmergencyScreen} from "../features/emergency";
import {InboxScreen} from "../features/inbox";
import {LibraryScreen} from "../features/library";
import {PdfDialogs} from "../features/pdf";
import {RestructureScreen} from "../features/restructure";
import {SplitDialog} from "../features/split";
import {SettingsScreen} from "../features/settings";
import {TaxScreen} from "../features/tax";
import {operations} from "../session/operations";
import {LIBRARY_QUERY, type SessionState, useSession} from "../session/session";
import {DialogHost, Spinner} from "../shared/components";
import {LibraryProblem} from "./library-problem";
import {NavigationSidebar} from "./navigation/navigation-sidebar";
import {CommandPalette} from "./palette/command-palette";
import {goTo, navigation, type Screen, setLogVisible, useNavigation} from "./screens";
import {ActivityBar} from "./status/activity-bar";
import {LogPanel} from "./status/log-panel";
import {Snackbar} from "./status/snackbar";
import {ScreenToolbar} from "./toolbar/screen-toolbar";

/** The navigation on the left, the screen, the log and the activity bar below it, and the messages. */
export function AppShell() {
  const state = useSession();
  const nav = useNavigation();
  const counts = useQuery({ queryKey: [LIBRARY_QUERY, "navigation-counts"], queryFn: () => commands.navigationCounts() });
  // Ask needs the AI: when the user switches AI off while it shows, the dashboard takes its place.
  useEffect(() => {
    if (!state.aiEnabled && nav.screen === "ask") goTo("dashboard");
  }, [state.aiEnabled, nav.screen]);
  return (
    <div className="app">
      <NavigationSidebar
        screen={nav.screen}
        hidden={state.aiEnabled ? [] : ["ask"]}
        counts={counts.data ?? { inbox: 0, tasks: 0, toPay: 0, cases: 0 }}
        // The library needs the width for its own sidebar, so the navigation is a rail there.
        collapsed={nav.collapsed || nav.screen === "library"}
        onToggleCollapsed={() => navigation.set((current) => ({ ...current, collapsed: !current.collapsed }))}
        onScreenChange={(screen) => goTo(screen)}
      />
      <main className="app-main">
        <div className="app-screen">
          <ScreenContent state={state} screen={nav.screen} />
        </div>
        {nav.logVisible && <LogPanel />}
        <ActivityBar logVisible={nav.logVisible} onToggleLog={() => setLogVisible(!nav.logVisible)} />
      </main>
      <Snackbar />
      <DialogHost />
    </div>
  );
}

/** The screen for the library's status: settings until it is configured, the chosen screen once it is open. */
function ScreenContent({ state, screen }: { state: SessionState; screen: Screen }) {
  switch (state.status.kind) {
    case "loading":
      return <div className="center-fill"><Spinner /></div>;
    case "notConfigured":
      return <SettingsScreen />;
    case "primaryFailed":
    case "failed":
      return screen === "settings" ? <SettingsScreen /> : <LibraryProblem status={state.status} onRetry={operations.openLibrary} onOpenSettings={() => goTo("settings")} />;
    case "open":
      return <OpenLibraryContent screen={screen} />;
  }
}

/** The chosen screen of an open library under the window toolbar, and the command palette. */
function OpenLibraryContent({ screen }: { screen: Screen }) {
  return (
    <div className="column" style={{ height: "100%" }}>
      <CommandPalette />
      <DocumentDialogs />
      <PdfDialogs />
      <BillsDialogs />
      <CaseDialogs />
      <SplitDialog />
      <ScreenToolbar screen={screen} />
      <div className="screen-body">
        {screen === "dashboard" && <DashboardScreen />}
        {screen === "inbox" && <InboxScreen />}
        {screen === "tasks" && <TasksScreen />}
        {screen === "library" && <LibraryScreen />}
        {screen === "ask" && <AskScreen />}
        {screen === "bills" && <BillsScreen />}
        {screen === "contracts" && <ContractsScreen />}
        {screen === "tax" && <TaxScreen />}
        {screen === "cases" && <CasesScreen />}
        {screen === "emergency" && <EmergencyScreen />}
        {screen === "restructure" && <RestructureScreen />}
        {screen === "settings" && <SettingsScreen />}
      </div>
    </div>
  );
}
