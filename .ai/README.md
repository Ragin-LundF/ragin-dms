# AI Instruction Index

This folder contains the canonical AI instructions for this repository.

The goal is progressive loading: read only what is needed for the current task instead of loading all instructions every time.

## Always read first

- `instructions/coding-guidelines.md` when creating, editing, reviewing, or testing Rust or TypeScript code.

## Load by task scope

### Repository architecture, crate boundaries, or the build

Read:

- `instructions/architecture-overview.md`

Use this for changes that affect crate boundaries, dependency direction, ports, module layout, the Cargo workspace, the npm/Tauri build, `scripts/fetch-pdfium.sh`, the CI workflows, or interactions between crates.

### Domain and use cases (`crates/domain`, `crates/application`)

Read:

- `instructions/architecture-overview.md`
- `instructions/domain-application.md`

Use these when touching files in or related to:

- `crates/domain` / `dms_domain::*`
- `crates/application` / `dms_application::*`
- ports, use cases, the services bundle, the catalog model, documents, categories, tags, library folders, detected changes, dashboard tasks

### Storage, catalog file, and synchronization (`crates/infra-storage`)

Read:

- `instructions/architecture-overview.md`
- `instructions/infra-storage.md`

Use these when touching files in or related to:

- `crates/infra-storage` / `dms_infra_storage::*`
- reading or writing user documents on disk
- the catalog file format, the YAML reader and emitter, or catalog migrations
- the library folders (one primary, any number of sync folders) and synchronizing them
- scanning folders and detecting changes for the dashboard
- the `.dms-text/` text store and the local state in the app data directory

If the change alters what the catalog stores, also read `instructions/domain-application.md`.

### AI providers and prompts (`crates/infra-ai`)

Read:

- `instructions/architecture-overview.md`
- `instructions/infra-ai.md`

Use these when touching files in or related to:

- `crates/infra-ai` / `dms_infra_ai::*`
- LLM providers (LM Studio, Ollama, OpenAI-compatible APIs, Anthropic), prompts, structured output, embeddings
- the page layout reader of "Convert to real PDF"

### PDF engine and document reading (`crates/infra-pdf`)

Read:

- `instructions/architecture-overview.md`
- `instructions/infra-pdf.md`

Use these when touching files in or related to:

- `crates/infra-pdf` / `dms_infra_pdf::*`
- reading documents for the AI and search: PDF text layer, rendered PDF pages, images, Office files
- thumbnails and page pictures, page tools, text layers, metadata, the rebuild of scans, `ocrmypdf`
- PDFium (`pdfium-render`, `scripts/fetch-pdfium.sh`) and `lopdf`

If the change touches what is sent to the AI (prompts, image sizes), also read `instructions/infra-ai.md`.

### Desktop app (`src-tauri`, `ui`)

Read:

- `instructions/architecture-overview.md`
- `instructions/desktop-ui.md`

Use these when touching files in or related to:

- `src-tauri`: commands, DTOs, presenters, the session, the menu, the `dms://` protocol, the composition root
- `ui`: React components, styles, the generated IPC bindings, the mocks
- the dashboard, the library screen, settings

If the UI change depends on use case behavior, also read `instructions/domain-application.md`.

### Tests

Read:

- `instructions/testing.md`

Use this when adding, modifying, or reviewing tests. Combine it with the file of the code under test, for example `infra-storage.md` + `testing.md`.

### Static analysis, clippy, rustfmt, or type errors

Read:

- `instructions/static-analysis.md`

Use this when fixing clippy, rustfmt or TypeScript findings, adding `#[allow]`s, or changing style-related code.

## Conflict resolution

If instructions conflict:

1. Prefer user instructions in the current chat or task.
2. Prefer the most specific `.ai/instructions/*.md` file over a general one.
3. Prefer crate-specific rules over repository-wide rules.
4. Preserve behavior before refactoring style.
5. Explain any unresolved conflict before changing architecture or behavior.

## Token-saving rule

Do not load every file in `.ai/instructions/` by default. Start with this index, then load only the files named above that match the current task.
