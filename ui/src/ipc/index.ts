import {convertFileSrc} from "@tauri-apps/api/core";
import type {IpcError} from "./bindings";

export * from "./bindings";

type Result<T> = { status: "ok"; data: T } | { status: "error"; error: IpcError };

/** Turns a command result into a value or a thrown [`IpcFailure`], which TanStack Query expects. */
export async function unwrap<T>(result: Promise<Result<T>>): Promise<T> {
  const settled = await result;
  if (settled.status === "error") throw new IpcFailure(settled.error);
  return settled.data;
}

export class IpcFailure extends Error {
  readonly code: string;

  constructor(error: IpcError) {
    super(error.message);
    this.code = error.code;
  }
}

/** A message for the user. Backend messages are already phrased in English; only broken calls are generic. */
export function describeError(error: unknown): string {
  if (error instanceof IpcFailure) return error.message;
  return error instanceof Error ? error.message : String(error);
}

/** URL of the thumbnail of the file at `path` (relative) in library folder `folderId`, served by `dms://`. */
export function thumbnailUrl(folderId: string, path: string): string {
  return convertFileSrc(`thumb/${folderId}/${path}`, "dms");
}

/** URL of one page of a document, `width` pixels wide. */
export function pageUrl(documentId: string, page: number, width: number): string {
  return `${convertFileSrc(`page/${documentId}/${page}`, "dms")}?width=${Math.round(width)}`;
}
