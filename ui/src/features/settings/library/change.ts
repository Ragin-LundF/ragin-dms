import { commands, unwrap, type LibrarySettingsChangeDto_Deserialize } from "../../../ipc";
import { reportError } from "../../../session/session";

/** Changes the catalog settings as "Saving the library settings"; Rust reports the outcome. */
export function changeLibrarySettings(change: LibrarySettingsChangeDto_Deserialize) {
  void unwrap(commands.changeLibrarySettings(change)).catch(reportError);
}
