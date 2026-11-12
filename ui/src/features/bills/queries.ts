import {useQuery} from "@tanstack/react-query";
import {type BillsFilterDto, commands, unwrap} from "../../ipc";
import {LIBRARY_QUERY, reportError} from "../../session/session";

export function useBillsView(filter: BillsFilterDto) {
  return useQuery({ queryKey: [LIBRARY_QUERY, "bills", filter], queryFn: () => unwrap(commands.billsView(filter)) });
}

export function useExpectedMissing() {
  return useQuery({ queryKey: [LIBRARY_QUERY, "bills", "expected"], queryFn: () => unwrap(commands.billsExpectedMissing()) });
}

export function markPaid(documentId: string) {
  void unwrap(commands.billsMarkPaid(documentId)).catch(reportError);
}

export function markUnpaid(documentId: string) {
  void unwrap(commands.billsMarkUnpaid(documentId)).catch(reportError);
}
