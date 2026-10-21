# AI Instruction Index

This folder contains the canonical AI instructions for this repository.

The goal is progressive loading: read only what is needed for the current task instead of loading all instructions every time.

## Always read first

- `instructions/coding-guidelines.md` when creating, editing, reviewing, or testing Kotlin code.

## Load by task scope

### Repository architecture, module boundaries, or Gradle build

Read:

- `instructions/architecture-overview.md`

Use this for changes that affect module boundaries, dependency direction, ports, package layout, the build setup in `config/gradle/`, or interactions between modules.

### Domain and use cases (`dms-domain`, `dms-application`)

Read:

- `instructions/architecture-overview.md`
- `instructions/dms-core.md`

Use these when touching files in or related to:

- `dms-domain` / `dms.domain.*`
- `dms-application` / `dms.application.*`
- ports, use cases, the catalog model, documents, categories, tags, library folders, detected changes, dashboard tasks

### Storage, catalog file, and synchronization (`dms-storage`)

Read:

- `instructions/architecture-overview.md`
- `instructions/dms-storage.md`

Use these when touching files in or related to:

- `dms-storage` / `dms.storage.*`
- reading or writing user documents on disk
- the catalog file format, its serialization, or its migration
- the library folders (one primary, any number of sync folders) and synchronizing them
- scanning folders and detecting changes for the dashboard
- the `.dms-text/` text store

If the change alters what the catalog stores, also read `instructions/dms-core.md`.

### AI classification and text extraction (`dms-ai`)

Read:

- `instructions/architecture-overview.md`
- `instructions/dms-ai.md`

Use these when touching files in or related to:

- `dms-ai` / `dms.ai.*`
- LLM providers (LM Studio, Ollama, OpenAI-compatible APIs), prompts, structured output
- reading documents: PDF text layer, rendered PDF pages, images, Office files (Apache POI)

### UI (`dms-ui`)

Read:

- `instructions/architecture-overview.md`
- `instructions/dms-ui.md`

Use these when touching files in or related to:

- `dms-ui`
- Compose UI code, view models, theme, the dashboard
- `commonMain` / `jvmMain` source sets
- the composition root that wires adapters to use cases

If the UI change depends on use case behavior, also read `instructions/dms-core.md`.

### Tests

Read:

- `instructions/testing.md`

Use this when adding, modifying, or reviewing tests. Combine it with the module file of the code under test, for example `dms-storage.md` + `testing.md`.

### Static analysis, Detekt, or lint cleanup

Read:

- `instructions/static-analysis.md`

Use this when fixing Detekt findings, adding suppressions, or changing style-related code.

### Executing the implementation plan

Read:

- `.plan/IMPLEMENTATION-PLAN.md`, the phase you are working on only

Then load the instruction files for the modules that phase touches.

## Conflict resolution

If instructions conflict:

1. Prefer user instructions in the current chat or task.
2. Prefer the most specific `.ai/instructions/*.md` file over a general one.
3. Prefer module-specific rules over repository-wide rules.
4. Preserve behavior before refactoring style.
5. Explain any unresolved conflict before changing architecture or behavior.

## Token-saving rule

Do not load every file in `.ai/instructions/` by default. Start with this index, then load only the files named above that match the current task.
