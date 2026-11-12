import type {EmergencyGroupDto} from "../../ipc";

/** The groups in the order of the overview, with their labels for menus. */
export const GROUPS: { group: EmergencyGroupDto; label: string }[] = [
  { group: "contacts", label: "Contacts" },
  { group: "insurance", label: "Insurance" },
  { group: "accounts", label: "Accounts" },
  { group: "contracts", label: "Contracts" },
  { group: "personal", label: "Personal" },
  { group: "other", label: "Other" },
];
