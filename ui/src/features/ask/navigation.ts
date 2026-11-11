import {goTo} from "../../shell/screens";

/** Asks the whole library `question` on the Ask screen (from the palette). */
export function askLibrary(question: string) {
  goTo("ask", { question });
}
