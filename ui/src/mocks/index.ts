import {mockConvertFileSrc, mockIPC, mockWindows} from "@tauri-apps/api/mocks";
import {HANDLERS} from "./handlers";

/**
 * Answers every command from synthetic data, for working in a browser (`npm run dev:mock`) and for headless
 * checks. Each feature adds its handlers in `handlers/<feature>.ts`; an unknown command fails like an unwired one.
 */
export function installMocks() {
  (window as { __DMS_MOCK__?: boolean }).__DMS_MOCK__ = true;
  mockWindows("main");
  mockConvertFileSrc("macos");
  mockIPC(
    async (command, args) => {
      if (command.startsWith("plugin:")) return pluginCommand(command, (args ?? {}) as { options?: { directory?: boolean } });
      const handler = HANDLERS[command];
      if (!handler) throw { code: "not_wired", message: `${command} is not mocked` };
      return handler((args ?? {}) as Record<string, unknown>);
    },
    { shouldMockEvents: true },
  );
}

/** File dialogs answer with a synthetic path; nothing on disk is touched. */
function pluginCommand(command: string, args: { options?: { directory?: boolean } }): unknown {
  if (command === "plugin:dialog|open") return args.options?.directory ? "/Synthetic/Documents" : ["/Synthetic/Inbox/scan-2026-01-04.pdf"];
  if (command === "plugin:dialog|save") return "/Synthetic/export.zip";
  return null;
}
