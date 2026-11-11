import {useEffect, useState} from "react";
import {keepPreviousData, useQuery} from "@tanstack/react-query";
import {commands, type LibraryFolderViewParamsDto, type LibraryStateDto_Serialize, unwrap} from "../../../ipc";
import {LIBRARY_QUERY} from "../../../session/session";
import {createStore} from "../../../shared/store";
import {paramsOf, type ViewState} from "./view-state";

export type LibraryState = LibraryStateDto_Serialize;
export type DocumentRow = LibraryState["groups"][number]["rows"][number];

/** The library state the screen shows now, for the actions, the keys and the menu, which act on what is shown. */
export const shownStore = createStore<LibraryState | null>(null);

/** The search text reads the documents' text once the user stopped typing for this long (Kotlin `ContentSearch`). */
const TYPING_PAUSE_MS = 300;

export function useLibraryState(view: ViewState) {
  const params = paramsOf(view);
  const typed = useDebounced(view.query.trim(), TYPING_PAUSE_MS);
  const contentQuery = view.contentSearch && typed ? typed : null;
  const query = useQuery({
    queryKey: [LIBRARY_QUERY, "library", "state", params, contentQuery],
    queryFn: () => unwrap(commands.libraryState(params, contentQuery)),
    placeholderData: keepPreviousData,
  });
  useEffect(() => {
    if (query.data) shownStore.set(query.data);
  }, [query.data]);
  return query;
}

/** The folders on disk, also empty ones; read again after every operation, kept while they cannot be read. */
export function useDiskFolders(): string[] {
  const query = useQuery({
    queryKey: [LIBRARY_QUERY, "library", "disk-folders"],
    queryFn: () => unwrap(commands.libraryDiskFolders()),
    placeholderData: keepPreviousData,
  });
  return query.data ?? [];
}

export function useFolderView(params: LibraryFolderViewParamsDto) {
  return useQuery({
    queryKey: [LIBRARY_QUERY, "library", "folder-view", params],
    queryFn: () => unwrap(commands.libraryFolderView(params)),
    placeholderData: keepPreviousData,
  });
}

export function useFolderTree(foldersOnDisk: string[], expanded: string[], enabled: boolean) {
  return useQuery({
    queryKey: [LIBRARY_QUERY, "library", "folder-tree", foldersOnDisk, expanded],
    queryFn: () => unwrap(commands.libraryFolderTree(foldersOnDisk, expanded)),
    placeholderData: keepPreviousData,
    enabled,
  });
}

function useDebounced<T>(value: T, delay: number): T {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setSettled(value), delay);
    return () => window.clearTimeout(timer);
  }, [value, delay]);
  return settled;
}
