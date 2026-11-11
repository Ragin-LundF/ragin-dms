import type {
    AiDraftDto,
    AiSettingsDto_Serialize,
    FolderRowDto_Serialize,
    InboxSettingsDto,
    LibraryCommandDto,
    LibrarySettingsChangeDto_Deserialize,
    LibrarySettingsDto
} from "../../ipc";
import {FIXTURE_FOLDERS} from "../fixture";
import type {Handler} from "./index";
import {mockOperation, mockSession, updateMockSession} from "./shell";

type MockFolder = Omit<FolderRowDto_Serialize, "note">;

const mockFolders: MockFolder[] = [
  { id: FIXTURE_FOLDERS.primary, displayName: "Documents", location: "/Synthetic/Documents", primary: true },
  { id: FIXTURE_FOLDERS.sync, displayName: "Documents", location: "/Synthetic/Cloud/Documents", primary: false },
];

let addNewFiles = true;
let aiSettings: AiSettingsDto_Serialize = {
  enabled: true,
  baseUrl: "http://localhost:1234/v1",
  model: null,
  preferVisionForPdfs: false,
  timeoutSeconds: 300,
  acceptSuggestedFileNames: false,
  embeddingModel: null,
  inputCharacters: 12000,
  provider: "local",
  anthropicModel: "claude-opus-5-5",
};

const TYPES = { text: "Text", number: "Number", money: "Amount", date: "Date", boolean: "Yes/no" } as const;
const ROLES = { dueDate: "Due date", cancelBy: "Cancel by", contractEnd: "Contract end", warrantyEnd: "Warranty end" } as const;

/** The catalog settings of the mocked library; the inbox mock reads its inbox from here. */
export const mockLibrarySettings: LibrarySettingsDto = {
  inbox: { enabled: true, path: "Inbox", aiAnalysis: true, aiTargetFolder: true, autoAnalyze: false },
  fields: [
    { key: "amount", label: "Amount", text: "Amount  (amount, Amount)" },
    { key: "due", label: "Due date", text: "Due date  (due, Date · deadline: Due date)" },
  ],
  fieldTypes: Object.entries(TYPES).map(([value, label]) => ({ value: value as keyof typeof TYPES, label })),
  fieldRoles: Object.entries(ROLES).map(([value, label]) => ({ value: value as keyof typeof ROLES, label })),
  defaultCurrency: "EUR",
  embedMetadata: false,
  rules: [
    { id: "r-1", name: "Northbank", title: "Northbank (learned)", description: "When from Northbank: category Bank statements, tags 2026", enabled: true },
    { id: "r-2", name: "Rule for City Power", title: "Rule for City Power (manual)", description: "When from City Power and name like *invoice*: category Home, tags invoice, folder Home", enabled: false },
  ],
};

function settingsState() {
  const status = mockSession.state.status;
  const folders = status.kind === "notConfigured" ? [] : mockFolders;
  return {
    configured: folders.length > 0,
    folders: folders.map((folder) => ({ ...folder, note: status.kind === "open" && status.unreachableFolders.includes(folder.id) ? "Not reachable when the library was opened" : null })),
    addNewFilesWhileScanning: addNewFiles,
    ai: aiSettings,
  };
}

function syncSessionFolders() {
  updateMockSession((state) => ({ ...state, folders: mockFolders.map(({ id, location, primary }) => ({ id, location, primary })) }));
}

function configure(command: LibraryCommandDto) {
  const name = (location: string) => location.replace(/[/\\]+$/, "").split(/[/\\]/).pop() || location;
  switch (command.kind) {
    case "createPrimary":
      mockFolders.splice(0, mockFolders.length, { id: FIXTURE_FOLDERS.primary, displayName: name(command.location), location: command.location, primary: true });
      updateMockSession((state) => ({ ...state, status: { kind: "open", foreignFolders: [], unreachableFolders: [] } }));
      break;
    case "addSyncFolder":
      if (mockFolders.some((folder) => folder.location === command.location)) throw { code: "invalid_input", message: "This folder is already part of the library" };
      mockFolders.push({ id: `f-${mockFolders.length + 1}`, displayName: name(command.location), location: command.location, primary: false });
      break;
    case "removeSyncFolder":
      mockFolders.splice(mockFolders.findIndex((folder) => folder.id === command.folderId), 1);
      break;
    case "choosePrimary":
      mockFolders.forEach((folder) => (folder.primary = folder.id === command.folderId));
      break;
  }
  syncSessionFolders();
  return mockOperation("Updating the library folders", "library", 3, "Opened the library");
}

function changeLibrarySettings(change: LibrarySettingsChangeDto_Deserialize) {
  const settings = mockLibrarySettings;
  let done: string;
  switch (change.kind) {
    case "addField": {
      const key = change.label.trim().toLowerCase().replace(/[\s_]+/g, "-").replace(/[^a-z0-9_-]/g, "");
      if (!key || settings.fields.some((field) => field.key === key)) {
        return void mockMessage("Choose another name for the field", true);
      }
      const role = change.fieldType === "date" && change.role ? ` · deadline: ${ROLES[change.role]}` : "";
      settings.fields.push({ key, label: change.label.trim(), text: `${change.label.trim()}  (${key}, ${TYPES[change.fieldType]}${role})` });
      done = `Added the field ${change.label.trim()}`;
      break;
    }
    case "removeField":
      settings.fields = settings.fields.filter((field) => field.key !== change.key);
      done = "Removed the field";
      break;
    case "addStandardFields":
      done = "Added the standard fields";
      break;
    case "saveCurrency": {
      const code = change.typed.trim().toUpperCase();
      if (code && !/^[A-Z]{3}$/.test(code)) return void mockMessage("Write a three-letter currency code such as EUR", true);
      settings.defaultCurrency = code;
      done = "Saved the default currency";
      break;
    }
    case "setEmbedMetadata":
      settings.embedMetadata = change.enabled;
      done = change.enabled ? "PDFs get the approved metadata" : "PDFs stay as they are";
      break;
    case "setRuleEnabled": {
      const rule = settings.rules.find((candidate) => candidate.id === change.ruleId);
      if (!rule) return null;
      rule.enabled = change.enabled;
      done = `Switched ${rule.name} ${change.enabled ? "on" : "off"}`;
      break;
    }
    case "removeRule":
      settings.rules = settings.rules.filter((rule) => rule.id !== change.ruleId);
      done = "Removed the rule";
      break;
  }
  return mockOperation("Saving the library settings", "library", 2, done);
}

function mockMessage(text: string, isError: boolean) {
  void import("@tauri-apps/api/event").then(({ emit }) => emit("message-event", { message: { text, isError } }));
}

/** A rough stand-in for Rust's draft check, only so the mocked card behaves. */
function checkAiDraft(draft: AiDraftDto, saved: AiSettingsDto_Serialize) {
  const url = draft.baseUrl.trim();
  const addressValid = draft.provider !== "local" || url.startsWith("http://") || url.startsWith("https://");
  const timeout = Number.parseInt(draft.timeoutSeconds, 10);
  const input = Number.parseInt(draft.inputCharacters, 10);
  const timeoutValid = timeout > 0;
  const inputValid = input >= 1000 && input <= 200000;
  const problem = !addressValid
    ? "Enter a server address starting with http:// or https://"
    : !timeoutValid
      ? "Enter a timeout above 0 seconds"
      : !inputValid
        ? "Enter a text size from 1000 to 200000 characters"
        : null;
  const blank = (text: string) => (text.trim() === "" ? null : text.trim());
  const settings: AiSettingsDto_Serialize | null = problem
    ? null
    : {
        ...draft,
        baseUrl: url.replace(/\/+$/, ""),
        model: blank(draft.model),
        embeddingModel: blank(draft.embeddingModel),
        timeoutSeconds: timeout,
        inputCharacters: input,
        anthropicModel: blank(draft.anthropicModel) ?? "claude-opus-5-5",
      };
  const saveDisabledReason = settings === null ? problem : JSON.stringify(settings) === JSON.stringify(saved) ? "Nothing changed" : null;
  return { addressValid, timeoutValid, inputValid, problem, settings, saveDisabledReason };
}

function checkInboxPath(path: string): string | null {
  const segments = path.split("/").map((segment) => segment.trim()).filter(Boolean);
  if (segments.length === 0) return "Name a folder inside the library, such as Inbox";
  if (segments.some((segment) => segment.startsWith("."))) return "The name must not start with a dot or a space";
  return null;
}

/** Mocked commands of features::settings. */
export const settingsHandlers: Record<string, Handler> = {
  settings_state: () => settingsState(),
  library_settings: () => (mockSession.state.status.kind === "open" ? mockLibrarySettings : null),
  configure_library: ({ command }) => configure(command as LibraryCommandDto),
  save_add_new_files_while_scanning: ({ enabled }) => {
    addNewFiles = enabled as boolean;
    updateMockSession((state) => ({ ...state, addNewFilesWhileScanning: addNewFiles }));
    return mockOperation("Saving the preferences", "library", 1, "Saved the preferences");
  },
  save_ai_settings: ({ settings }) => {
    aiSettings = settings as AiSettingsDto_Serialize;
    updateMockSession((state) => ({ ...state, aiEnabled: aiSettings.enabled }));
    return mockOperation("Saving the preferences", "library", 1, "Saved the preferences");
  },
  check_ai_draft: ({ draft, saved }) => checkAiDraft(draft as AiDraftDto, saved as AiSettingsDto_Serialize),
  test_ai_connection: async ({ settings }) => {
    await new Promise((resolve) => window.setTimeout(resolve, 600));
    const models = (settings as AiSettingsDto_Serialize).provider === "anthropic" ? ["claude-opus-5-5", "claude-sonnet-5"] : ["qwen2.5-vl-7b-instruct", "gemma-3-12b", "text-embedding-nomic"];
    return { models, message: `Connected. The server offers ${models.length} models`, isError: false };
  },
  change_library_settings: ({ change }) => changeLibrarySettings(change as LibrarySettingsChangeDto_Deserialize),
  check_inbox_path: ({ path }) => checkInboxPath(path as string),
  save_inbox: ({ inbox }) => {
    mockLibrarySettings.inbox = inbox as InboxSettingsDto;
    const saved = inbox as InboxSettingsDto;
    return mockOperation("Saving the inbox settings", "library", 2, saved.enabled ? `The inbox is ${saved.path} in every library folder` : "The inbox is off");
  },
  /** Mock only: switches the mocked library's status, e.g. `{ kind: "notConfigured" }`, for screenshots. */
  mock_set_status: ({ status }) => {
    updateMockSession((state) => ({ ...state, status: status as typeof state.status }));
    return null;
  },
};
