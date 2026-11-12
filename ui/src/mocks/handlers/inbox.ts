import type {InboxStateDto_Serialize} from "../../ipc";
import {fileName, FIXTURE_DIRECTORIES, FIXTURE_DOCUMENTS} from "../fixture";
import type {Handler} from "./index";
import {mockOperation, mockSession} from "./shell";
import {mockLibrarySettings} from "./settings";

let recentTargets: string[] = [];

function inboxState(): InboxStateDto_Serialize {
  const inbox = mockLibrarySettings.inbox;
  const prefix = `${inbox.path}/`;
  const rows = inbox.enabled
    ? FIXTURE_DOCUMENTS.filter((document) => document.path.startsWith(prefix))
        .sort((first, second) => second.addedAt.localeCompare(first.addedAt))
        .map((document) => {
          const parent = document.path.slice(0, document.path.lastIndexOf("/"));
          return {
            documentId: document.id,
            name: fileName(document.path),
            subfolder: parent === inbox.path ? "" : parent.slice(prefix.length),
            addedAt: document.addedAt,
            size: document.size,
            hasSuggestion: document.suggestion,
            suggestionNote: document.suggestion ? (document.correspondent ? " · AI suggests Home/Bills" : " · AI suggestion waiting") : "",
            analyzable: document.kind !== "other",
            prefill: null,
            email: document.path.toLowerCase().endsWith(".eml"),
          };
        })
    : [];
  return {
    enabled: inbox.enabled,
    path: inbox.path,
    aiAnalysis: inbox.aiAnalysis && mockSession.state.aiEnabled,
    rows,
    folders: FIXTURE_DIRECTORIES.filter((folder) => folder !== inbox.path && !folder.startsWith(prefix)).sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase())),
    recentTargets,
  };
}

function document(id: unknown) {
  const found = FIXTURE_DOCUMENTS.find((candidate) => candidate.id === id);
  if (!found) throw { code: "invalid_input", message: "The document or file no longer exists" };
  return found;
}

/** Mocked commands of features::inbox. */
export const inboxHandlers: Record<string, Handler> = {
  inbox_state: () => inboxState(),
  analyze_inbox_document: ({ documentId }) => {
    const found = document(documentId);
    const name = fileName(found.path);
    mockOperation(`Analyzing ${name}`, "ai", 4, `The AI suggested a classification for ${name}`);
    found.suggestion = true;
    return null;
  },
  inbox_import_emails: ({ documentIds }) => {
    const count = (documentIds as string[]).length;
    mockOperation(`Importing ${count} email(s)`, "library", 3, `Imported ${count} email(s) with 2 attachment(s)`);
    return null;
  },
  analyze_inbox: () => mockOperation("Analyzing the inbox", "ai", 6, "Analyzing the inbox: finished", true),
  file_inbox_document: ({ documentId, folder, fileName: typedName }) => {
    if (mockSession.state.operations.some((operation) => operation.lane === "library")) return false;
    const found = document(documentId);
    const name = fileName(found.path);
    const target = (folder as string).split("/").map((segment) => segment.trim()).filter(Boolean).join("/");
    mockOperation(`Filing ${name}`, "library", 2, `Filed ${name} in ${target}`);
    found.path = `${target}/${(typedName as string).trim() || name}`;
    recentTargets = [target, ...recentTargets.filter((recent) => recent !== target)].slice(0, 5);
    return true;
  },
};
