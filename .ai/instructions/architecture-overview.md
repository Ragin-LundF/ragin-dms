# Architecture Overview

Read this file for repository-level architecture decisions, cross-crate changes, changes that affect crate boundaries, and changes to the build.

## What the application is

A desktop document management system that keeps documents as **ordinary, readable files on disk**. The tool adds documents, creates folders, and stores categorization (categories, tags, metadata) in a **catalog file** in the root of the document library. Nothing is hidden in a database: deleting the tool must leave a complete, browsable folder tree behind.

The library exists in **one primary folder and any number of sync folders** configured as plain **paths** per device. What is behind a path (local disk, external disk, network share, a folder a cloud client backs up) does not matter. The DMS synchronizes only between these paths and never talks to a cloud vendor; cloud backup is the vendor client's job. Every folder holds the same tree and its own copy of the catalog file. The tool writes every change to all of them, detects changes made outside the tool by scanning, and shows everything that needs a decision as an open task on the **dashboard**.

The app runs on macOS, Windows and Linux. Never assume one platform's file system behavior (case sensitivity, Unicode normalization, path separators, allowed characters).

## Clean Architecture crates

The repository is one Cargo workspace (`Cargo.toml`: members `crates/*` and `src-tauri`) plus the React webview in `ui/`.

| Crate | Layer | Holds | May depend on |
|---|---|---|---|
| `crates/domain` (`dms-domain`) | Entities | Entities, value objects, domain rules: `Document`, `Catalog`, `Category`, `TagName`, `LibraryFolder`, `DetectedChange`, `OpenTask`, typed ids and paths. | small pure crates only (`thiserror`, `chrono`, `chrono-tz`, `indexmap`, `unicode-normalization`, `num-bigint`) |
| `crates/application` (`dms-application`) | Use cases | Use cases (`AddDocuments`, `CreateFolder`, `SuggestClassification`, …), the `Replicator` that applies every write to all library folders (primary first, journal for failed sync folders), the **ports** they need (`DocumentStorage`, `CatalogRepository`, `DocumentClassifier`, `DocumentContentReader`, …) in `port::*`, and `services::AppServices`, the bundle of all use cases built from `AppPorts`. | `dms-domain` |
| `crates/infra-storage` (`dms-infra-storage`) | Adapter | File system access in one folder per call, the catalog file (own YAML reader and emitter), scanning, the `.dms-text/` store, local state (journal, tasks, activity, settings), export, vector index, plans, folder watcher, system trash. Implements the storage, state, export, index and plan ports. | `dms-application` |
| `crates/infra-ai` (`dms-infra-ai`) | Adapter | `ureq` client for key-less OpenAI-compatible APIs (LM Studio, Ollama) and Anthropic, prompts (`prompts/*.md`), structured output, embeddings, the page layout reader for the rebuild. Implements `DocumentClassifier`, `AiServer`, `DocumentAssistant`, `LibraryAssistant`, `EmbeddingClient`, `PageLayoutReader`. | `dms-application` |
| `crates/infra-pdf` (`dms-infra-pdf`) | Adapter | PDFium (`pdfium-render`) and `lopdf`: reading documents as text or page images (PDF, images, Office via `zip`/`quick-xml`/`calamine`), thumbnails and page pictures, page tools (rotate, delete, reorder, extract, merge), invisible text layers, embedded metadata (document info and XMP), the rebuild of scans, and the installed OCR program (`ocrmypdf`). Implements `DocumentContentReader`, `DocumentPreviewer`, `PdfEditor`, `PdfRebuilder`, `OcrTool`. | `dms-application` |
| `src-tauri` (`ragin-dms-desktop`) | Frameworks & drivers | The Tauri shell: commands, DTOs and presenters per screen area, the session (lanes, progress, log), outcome messages, menu, the `dms://` preview protocol, and the **composition root** (`context/composition.rs`) that wires the adapters into `AppServices`. | all crates above |
| `ui` | Frameworks & drivers | React 19 + TypeScript (Vite, TanStack Query). Renders what Rust computes and keeps only presentation state. Talks to Rust only through the generated bindings in `ui/src/ipc/bindings.ts`. | `src-tauri` commands and events |

## Dependency rule

Dependencies point inwards only:

```text
ui ──(IPC)──► src-tauri  ──►  dms-application  ──►  dms-domain
infra-storage ──────────────────┘   ▲
infra-ai ───────────────────────────┤
infra-pdf ──────────────────────────┘
```

- `dms-domain` does **no I/O** and must stay that way: no file system, no serde, no threads, no logging.
- `dms-application` knows ports (traits), never their implementations. It never uses `std::fs`, serde, `ureq`, PDFium or any AI library.
- Adapters (`infra-storage`, `infra-ai`, `infra-pdf`) never depend on each other and never on `src-tauri`.
- Only the composition root in `src-tauri` knows concrete adapter types (`StorageAdapters`, `AiAdapters`, `PdfAdapters`).
- A new external crate goes into the outermost crate that needs it. If a use case seems to need it, add a port instead. The crate graph enforces this: an import that points outwards does not compile, so never "fix" it by adding a dependency inwards.

## Ports and adapters

- A port is a trait in `dms_application::port::<area>` (`storage`, `state`, `ai`, `assistant`, `pdf`, `export`, `index`, `plan`, `system`), named after the capability (`DocumentStorage`, `DocumentClassifier`), not after a technology. Every port is `Send + Sync`.
- Port signatures use domain and port types only. No `std::path::Path`, `File`, HTTP or JSON types cross a port.
- Port functions are blocking `fn(&self, …)`. The desktop app calls use cases on a worker thread (`tauri::async_runtime::spawn_blocking` or a session operation); use cases never spawn threads themselves.
- Expected failures cross a port as a value (`StorageResult<T>` = `Result<T, StorageError>`, or an outcome enum), never as a panic. Panics are for broken invariants only (`expect("invariant: …")`).
- Long runs take `&dyn common::Progress<P>` and check `is_cancelled()` between items; a cancelled run stops before the next item and returns what it did so far.

## Data flow: from a new file to a categorized document

```text
ScanLibrary (all folders)            or   UI drop / file picker -> AddDocuments
  -> DetectedChange::NewFile                    -> DocumentStorage::import_file (all folders)
  -> BuildDashboard -> OpenTask::AdoptNewFile
User: "Adopt & suggest"
  -> SuggestClassification
     -> DocumentContentReader::read    (text layer, page images, or Office text)
     -> TextStore::write               (.dms-text/<documentId>.txt)
     -> DocumentClassifier::suggest    (suggestion only)
  -> OpenTask::ReviewSuggestion on the dashboard
User: approve (all or selected fields, edited)
  -> ApproveSuggestion
     -> DocumentStorage::move_entry    (all folders, if a new name was approved)
     -> CatalogRepository::save        (all folders, revision + 1)
```

AI output is always a **suggestion**. Nothing the AI produces reaches the catalog or a file name without the user's approval.

## Build

- `Cargo.toml` in the root declares the workspace, the shared version (`workspace.package`), shared dependency versions (`workspace.dependencies`) and the lints (`workspace.lints`). Crates take shared dependencies with `x.workspace = true` and declare `[lints] workspace = true`. Never weaken the workspace lints in a crate.
- The root `package.json` holds the Tauri CLI (`npm run dev`, `npm run build`); `ui/package.json` the webview (`dev`, `dev:mock`, `build`, `typecheck`, `test`).
- Tauri configuration: `src-tauri/tauri.conf.json` plus one file per platform (`tauri.macos.conf.json`, `tauri.macos-x64.conf.json`, `tauri.windows.conf.json`, `tauri.linux.conf.json`), which bundle PDFium from `vendor/pdfium/<platform>/`.
- PDFium is not built: `scripts/fetch-pdfium.sh` downloads a pinned release with checksums into the git-ignored `vendor/pdfium/`. The release must match the `pdfium-render` feature (`pdfium_8125`); change both together.
- `rustfmt.toml` sets the line width (120). CI (`.github/workflows/ci.yml`) runs formatting, clippy, tests and the UI checks; `release.yml` builds the installers. Shared IntelliJ run configurations are in `.run/`.
- The app data directory can be redirected with `RAGIN_DMS_DATA_DIR` for development; the default paths are the 1.x paths and must not change.
