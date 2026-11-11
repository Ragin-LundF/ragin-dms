import { useState, type ReactNode } from "react";
import type { LibrarySettingsDto, SettingsStateDto_Serialize } from "../../ipc";
import { ScreenHeader } from "../../shared/components";
import { AiSettingsCard } from "./ai/ai-card";
import { FoldersCard, WelcomeCard } from "./folders-card";
import { FieldsCard } from "./library/fields-card";
import { InboxCard } from "./library/inbox-card";
import { RulesCard } from "./library/rules-card";
import { useBusyReason, useLibrarySettings, useSettingsState } from "./queries";
import { ScanningCard } from "./scanning-card";
import { SECTIONS, SectionList, type SettingsSection } from "./section-list";
import "./settings.css";

/**
 * The library folders of this device, how scans treat new files, and where the AI runs. A configured library shows
 * one section at a time; until one is configured, the welcome card and the AI card.
 */
export function SettingsScreen() {
  const state = useSettingsState();
  const busyReason = useBusyReason();
  return (
    <div className="settings-screen">
      <ScreenHeader title="Settings" subtitle="Where your library lives on this device" />
      {state.data?.configured ? (
        <SectionedSettings state={state.data} busyReason={busyReason} />
      ) : (
        <div className="settings-column scroll">
          <WelcomeCard busyReason={busyReason} />
          {state.data && <AiSettingsCard settings={state.data.ai} busyReason={busyReason} />}
        </div>
      )}
    </div>
  );
}

// The chosen section survives leaving the screen, like Kotlin's rememberSaveable, not an app restart.
let chosenSection: SettingsSection = "folders";

function SectionedSettings({ state, busyReason }: { state: SettingsStateDto_Serialize; busyReason: string | null }) {
  const library = useLibrarySettings().data ?? null;
  const [chosen, setChosen] = useState(chosenSection);
  const sections = SECTIONS.filter((section) => library !== null || !section.needsLibrarySettings);
  const selected = sections.some((section) => section.id === chosen) ? chosen : (sections[0]?.id ?? "folders");
  const choose = (section: SettingsSection) => {
    chosenSection = section;
    setChosen(section);
  };
  return (
    <div className="settings-sectioned">
      <SectionList sections={sections} selected={selected} onSelect={choose} />
      {/* Every card stays mounted, hidden while another is chosen, so its unsaved draft waits for the user. */}
      {sections.map((section) => (
        <div key={section.id} className="settings-column scroll grow" hidden={section.id !== selected}>
          {card(section.id, state, library, busyReason)}
        </div>
      ))}
    </div>
  );
}

function card(section: SettingsSection, state: SettingsStateDto_Serialize, library: LibrarySettingsDto | null, busyReason: string | null): ReactNode {
  switch (section) {
    case "folders":
      return <FoldersCard rows={state.folders} busyReason={busyReason} />;
    case "scanning":
      return <ScanningCard addNewFiles={state.addNewFilesWhileScanning} busyReason={busyReason} />;
    case "inbox":
      return library && <InboxCard inbox={library.inbox} aiEnabled={state.ai.enabled} busyReason={busyReason} />;
    case "fields":
      return library && <FieldsCard settings={library} busyReason={busyReason} />;
    case "rules":
      return library && <RulesCard rules={library.rules} busyReason={busyReason} />;
    case "ai":
      return <AiSettingsCard settings={state.ai} busyReason={busyReason} />;
  }
}
