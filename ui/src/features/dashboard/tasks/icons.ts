import ACCOUNT_TREE from "@material-design-icons/svg/round/account_tree.svg?raw";
import ARROW_FORWARD from "@material-design-icons/svg/round/arrow_forward.svg?raw";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import AUTORENEW from "@material-design-icons/svg/round/autorenew.svg?raw";
import CHECK from "@material-design-icons/svg/round/check.svg?raw";
import CONTENT_COPY from "@material-design-icons/svg/round/content_copy.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import DOCUMENT_SCANNER from "@material-design-icons/svg/round/document_scanner.svg?raw";
import EDIT from "@material-design-icons/svg/round/edit.svg?raw";
import EMERGENCY from "@material-design-icons/svg/round/emergency.svg?raw";
import ERROR_OUTLINE from "@material-design-icons/svg/round/error_outline.svg?raw";
import EVENT from "@material-design-icons/svg/round/event.svg?raw";
import EVENT_BUSY from "@material-design-icons/svg/round/event_busy.svg?raw";
import FIND_IN_PAGE from "@material-design-icons/svg/round/find_in_page.svg?raw";
import FORUM from "@material-design-icons/svg/round/forum.svg?raw";
import HEALING from "@material-design-icons/svg/round/healing.svg?raw";
import HEALTH_AND_SAFETY from "@material-design-icons/svg/round/health_and_safety.svg?raw";
import INVENTORY from "@material-design-icons/svg/round/inventory_2.svg?raw";
import MERGE_TYPE from "@material-design-icons/svg/round/merge_type.svg?raw";
import MOVE_TO_INBOX from "@material-design-icons/svg/round/move_to_inbox.svg?raw";
import NEW_RELEASES from "@material-design-icons/svg/round/new_releases.svg?raw";
import OPEN_IN_NEW from "@material-design-icons/svg/round/open_in_new.svg?raw";
import PAYMENTS from "@material-design-icons/svg/round/payments.svg?raw";
import REFRESH from "@material-design-icons/svg/round/refresh.svg?raw";
import RESTORE from "@material-design-icons/svg/round/restore.svg?raw";
import RULE from "@material-design-icons/svg/round/rule.svg?raw";
import SWAP_HORIZ from "@material-design-icons/svg/round/swap_horiz.svg?raw";
import SYNC from "@material-design-icons/svg/round/sync.svg?raw";
import SYNC_PROBLEM from "@material-design-icons/svg/round/sync_problem.svg?raw";

/** The material icons Rust names in buttons and rows (snake case, as in Compose's `Icons.Rounded`). */
const ICONS: Record<string, string> = {
  account_tree: ACCOUNT_TREE,
  arrow_forward: ARROW_FORWARD,
  auto_awesome: AUTO_AWESOME,
  autorenew: AUTORENEW,
  check: CHECK,
  content_copy: CONTENT_COPY,
  delete: DELETE,
  document_scanner: DOCUMENT_SCANNER,
  edit: EDIT,
  emergency: EMERGENCY,
  error_outline: ERROR_OUTLINE,
  event: EVENT,
  event_busy: EVENT_BUSY,
  find_in_page: FIND_IN_PAGE,
  forum: FORUM,
  healing: HEALING,
  health_and_safety: HEALTH_AND_SAFETY,
  inventory_2: INVENTORY,
  merge_type: MERGE_TYPE,
  move_to_inbox: MOVE_TO_INBOX,
  new_releases: NEW_RELEASES,
  open_in_new: OPEN_IN_NEW,
  payments: PAYMENTS,
  refresh: REFRESH,
  restore: RESTORE,
  rule: RULE,
  swap_horiz: SWAP_HORIZ,
  sync: SYNC,
  sync_problem: SYNC_PROBLEM,
};

export function iconOf(name: string | null | undefined): string | undefined {
  return name ? ICONS[name] : undefined;
}
