import {goTo} from "../../shell/screens";

/** Opens the restructure assistant on `folder` (relative path; "" is the root), as a dashboard task asks. */
export function openRestructure(folder: string) {
  goTo("restructure", { folder });
}
