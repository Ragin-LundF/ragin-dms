import {commands, unwrap} from "../../ipc";
import {reportUnlessBusy} from "./actions";

/**
 * The AI's batch runs started from the dashboard, toolbar and palette (Kotlin `AiBatches`): tag untagged and
 * classify uncategorized documents, in the AI lane. Rust runs them and reports their progress and result.
 */
export const aiBatches = {
  tagAll: () => void unwrap(commands.aiTagAll()).catch(reportUnlessBusy),
  classifyAll: () => void unwrap(commands.aiClassifyAll()).catch(reportUnlessBusy),
};
