import {askHandlers} from "./ask";
import {dashboardHandlers} from "./dashboard";
import {documentHandlers} from "./document";
import {inboxHandlers} from "./inbox";
import {libraryHandlers} from "./library";
import {pdfHandlers} from "./pdf";
import {restructureHandlers} from "./restructure";
import {settingsHandlers} from "./settings";
import {shellHandlers} from "./shell";

/** A mocked command: gets the command's arguments by name, returns its result or throws an `IpcError`. */
export type Handler = (args: Record<string, unknown>) => unknown;

export const HANDLERS: Record<string, Handler> = {
  ...shellHandlers,
  ...dashboardHandlers,
  ...libraryHandlers,
  ...documentHandlers,
  ...settingsHandlers,
  ...inboxHandlers,
  ...restructureHandlers,
  ...askHandlers,
  ...pdfHandlers,
};
