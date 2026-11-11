/** Whether the page runs inside the Tauri window (and not in a browser with mocked commands). */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window && !(window as { __DMS_MOCK__?: boolean }).__DMS_MOCK__;
}
