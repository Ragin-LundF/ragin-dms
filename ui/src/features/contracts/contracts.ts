import {useQuery} from "@tanstack/react-query";
import {commands, unwrap} from "../../ipc";
import {LIBRARY_QUERY, reportError} from "../../session/session";
import {goTo} from "../../shell/screens";
import {openLibrary} from "../library/navigation";

/** What another screen asks the contracts screen: show a contract (and draft its cancellation letter), or create one. */
export type ContractsRequest =
  | { kind: "open"; contractId: string; draftLetter: boolean }
  | { kind: "create"; correspondent: string; documentIds: string[] };

export function openContract(contractId: string, draftLetter = false) {
  goTo("contracts", { kind: "open", contractId, draftLetter } satisfies ContractsRequest);
}

export function createContract(correspondent: string, documentIds: string[]) {
  goTo("contracts", { kind: "create", correspondent, documentIds } satisfies ContractsRequest);
}

/** Opens the library with the documents of `correspondent`. */
export function showDocumentsOf(correspondent: string) {
  openLibrary({ kind: "show", selection: { kind: "folder", path: "" }, query: "", filters: [{ kind: "correspondent", name: correspondent }] });
}

export function useContractsView() {
  return useQuery({ queryKey: [LIBRARY_QUERY, "contracts"], queryFn: () => unwrap(commands.contractsView()) });
}

export function useContractsStatus() {
  return useQuery({ queryKey: [LIBRARY_QUERY, "contracts", "status"], queryFn: () => unwrap(commands.contractsStatus()) });
}

export function useContractsOfDocument(documentId: string) {
  return useQuery({
    queryKey: [LIBRARY_QUERY, "contracts", "document", documentId],
    queryFn: () => unwrap(commands.contractsOfDocument(documentId)),
  });
}

export function attachToContract(contractId: string, documentIds: string[], detach = false): Promise<void> {
  return unwrap(commands.contractsAttach(contractId, documentIds, detach)).then(() => undefined, reportError);
}
