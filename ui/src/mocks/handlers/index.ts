import {askHandlers} from "./ask";
import {billHandlers} from "./bills";
import {contractHandlers} from "./contracts";
import {caseHandlers} from "./cases";
import {dashboardHandlers} from "./dashboard";
import {documentHandlers} from "./document";
import {healthHandlers} from "./health";
import {emergencyHandlers} from "./emergency";
import {inboxHandlers} from "./inbox";
import {letterHandlers} from "./letters";
import {libraryHandlers} from "./library";
import {pdfHandlers} from "./pdf";
import {restructureHandlers} from "./restructure";
import {settingsHandlers} from "./settings";
import {shellHandlers} from "./shell";
import {taxHandlers} from "./tax";
import {splitHandlers} from "./split";

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
  ...letterHandlers,
  ...taxHandlers,
  ...billHandlers,
  ...contractHandlers,
  ...caseHandlers,
  ...healthHandlers,
  ...emergencyHandlers,
  ...splitHandlers,
};
