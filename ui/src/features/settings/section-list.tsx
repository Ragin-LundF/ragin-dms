import STORAGE from "@material-design-icons/svg/round/storage.svg?raw";
import TRAVEL_EXPLORE from "@material-design-icons/svg/round/travel_explore.svg?raw";
import MOVE_TO_INBOX from "@material-design-icons/svg/round/move_to_inbox.svg?raw";
import TEXT_FIELDS from "@material-design-icons/svg/round/text_fields.svg?raw";
import RULE from "@material-design-icons/svg/round/rule.svg?raw";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import HEALTH from "@material-design-icons/svg/round/health_and_safety.svg?raw";
import {Icon} from "../../shared/components";

export type SettingsSection = "folders" | "scanning" | "inbox" | "fields" | "rules" | "health" | "ai";

/** The sections of a configured library, in order; those that edit the catalog need its settings loaded. */
export const SECTIONS: { id: SettingsSection; label: string; icon: string; needsLibrarySettings: boolean }[] = [
  { id: "folders", label: "Library folders", icon: STORAGE, needsLibrarySettings: false },
  { id: "scanning", label: "Scanning", icon: TRAVEL_EXPLORE, needsLibrarySettings: false },
  { id: "inbox", label: "Inbox", icon: MOVE_TO_INBOX, needsLibrarySettings: true },
  { id: "fields", label: "Custom fields", icon: TEXT_FIELDS, needsLibrarySettings: true },
  { id: "rules", label: "Matching rules", icon: RULE, needsLibrarySettings: true },
  { id: "health", label: "Archive health", icon: HEALTH, needsLibrarySettings: false },
  { id: "ai", label: "AI", icon: AUTO_AWESOME, needsLibrarySettings: false },
];

/** The settings sections as tabs, styled like the app navigation; the selected one sits on a surface pill. */
export function SectionList({ sections, selected, onSelect }: { sections: typeof SECTIONS; selected: SettingsSection; onSelect: (section: SettingsSection) => void }) {
  return (
    <div className="settings-sections" role="tablist" aria-orientation="vertical">
      {sections.map((section) => (
        <button
          key={section.id}
          type="button"
          role="tab"
          aria-selected={section.id === selected}
          className={section.id === selected ? "settings-section selected" : "settings-section"}
          onClick={() => onSelect(section.id)}
        >
          <Icon svg={section.icon} />
          <span className="label-large ellipsis">{section.label}</span>
        </button>
      ))}
    </div>
  );
}
