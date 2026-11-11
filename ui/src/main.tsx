import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "./styles/theme.css";
import "./styles/components.css";
import "./styles/shell.css";
import { connectSession } from "./session/session";
import { AppShell } from "./shell/app-shell";
import { connectMenu } from "./shell/menu-actions";

const queries = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false, refetchOnWindowFocus: false } } });

async function start() {
  // Outside Tauri (a browser, headless checks) every command is answered by the mocks.
  if (import.meta.env.MODE === "mock" || !("__TAURI_INTERNALS__" in window)) {
    const { installMocks } = await import("./mocks");
    installMocks();
  }
  if (!navigator.userAgent.includes("Mac")) document.documentElement.style.setProperty("--title-bar-inset", "0px");
  await Promise.all([connectSession(queries), connectMenu()]);
  const root = document.getElementById("root");
  if (!root) throw new Error("index.html has no #root");
  createRoot(root).render(
    <StrictMode>
      <QueryClientProvider client={queries}>
        <AppShell />
      </QueryClientProvider>
    </StrictMode>,
  );
}

void start();
