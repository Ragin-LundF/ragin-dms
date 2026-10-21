# Architecture Overview

Read this file for repository-level architecture decisions, cross-module changes, changes that affect module boundaries, and changes to the Gradle build.

## What the application is

A desktop document management system that keeps documents as **ordinary, readable files on disk**. The tool adds documents, creates folders, and stores categorization (categories, tags, metadata) in a **catalog file** in the root of the document library. Nothing is hidden in a database: deleting the tool must leave a complete, browsable folder tree behind.

The library exists in **one primary folder and any number of sync folders** configured as plain **paths** per device. What is behind a path (local disk, external disk, network share, a folder a cloud client backs up) does not matter. The DMS synchronizes only between these paths and never talks to a cloud vendor; cloud backup is the vendor client's job. Every folder holds the same tree and its own copy of the catalog file. The tool writes every change to all of them, detects changes made outside the tool by scanning, and shows everything that needs a decision as an open task on the **dashboard**.

The app runs on macOS, Windows and Linux. Never assume one platform's file system behavior (case sensitivity, Unicode normalization, path separators, allowed characters).

## Clean Architecture modules

| Module | Layer | Holds | May depend on |
|---|---|---|---|
| `dms-domain` | Entities | Entities, value objects, domain rules: `Document`, `Catalog`, `Category`, `Tag`, `LibraryFolder`, `DetectedChange`, `OpenTask`, typed ids and paths. | nothing |
| `dms-application` | Use cases | Use cases (`AddDocuments`, `CreateFolder`, `SuggestClassification`, …), the `Replicator` that applies every write to all library folders (primary first, journal for failed sync folders), and the **ports** they need (`DocumentStorage`, `CatalogRepository`, `DocumentClassifier`, `DocumentContentReader`, …). | `dms-domain`, kotlinx-coroutines |
| `dms-storage` | Adapter | File system access in one folder per call, the catalog file (de)serialization, scanning, the `.dms-text/` store, the journal file. Implements storage ports. | `dms-application` |
| `dms-ai` | Adapter | LLM client for OpenAI-compatible APIs (LM Studio, Ollama, OpenAI), prompts, reading documents as text or page images (PDFBox, Apache POI). Implements `DocumentClassifier` and `DocumentContentReader`. | `dms-application` |
| `dms-ui` | Frameworks & drivers | Compose Multiplatform desktop UI (single `jvm()` target), view models, theme, and the **composition root** that wires adapters into use cases. | `commonMain`: `dms-domain`, `dms-application`. `jvmMain`: also `dms-storage`, `dms-ai`. |

## Dependency rule

Dependencies point inwards only:

```text
dms-ui  ──►  dms-application  ──►  dms-domain
dms-storage ─┘        ▲
dms-ai ───────────────┘
```

- `dms-domain` has **no dependencies** and must keep none: no I/O, no serialization annotations, no coroutines, no logging.
- `dms-application` knows ports (interfaces), never their implementations. It never imports `java.nio.file`, Jackson, Ktor, or any AI library.
- Adapters (`dms-storage`, `dms-ai`) never depend on each other and never on `dms-ui`.
- Only the composition root in `dms-ui` `jvmMain` knows concrete adapter classes.
- A new external library goes into the outermost module that needs it. If a use case seems to need it, add a port instead.

## Ports and adapters

- A port is an interface in `dms.application.port`, named after the capability (`DocumentStorage`, `DocumentClassifier`), not after a technology.
- Port signatures use domain types only. No `java.nio.file.Path`, `File`, HTTP or JSON types cross a port.
- Port functions doing I/O are `suspend`. Adapters switch to `Dispatchers.IO` themselves; use cases do not pick dispatchers.
- Expected failures cross a port as a typed result (`sealed interface` of outcomes), not as an exception. Exceptions are for programming errors and truly unrecoverable state.

## Data flow: from a new file to a categorized document

```text
ScanLibrary (all folders)            or   UI drop / file picker -> AddDocuments
  -> DetectedChange.NewFile                     -> DocumentStorage.import (all folders)
  -> BuildDashboard -> OpenTask.ClassifyNewFile
User: "Adopt & suggest"
  -> SuggestClassification
     -> DocumentContentReader.read     (text layer, page images, or Office text)
     -> TextStore.write                (.dms-text/<documentId>.txt)
     -> DocumentClassifier.suggest     (suggestion only)
  -> OpenTask.ReviewSuggestion on the dashboard
User: approve (all or selected fields, edited)
  -> ApproveSuggestion
     -> DocumentStorage.rename         (all folders, if a new name was approved)
     -> CatalogRepository.save         (all folders, revision + 1)
```

AI output is always a **suggestion**. Nothing the AI produces reaches the catalog or a file name without the user's approval.

## Gradle build

The build follows the structure of the `rule-engine` project:

- `settings.gradle` declares the modules and repositories; the root `build.gradle` declares plugins with `apply false` and applies shared scripts to all subprojects.
- Shared build logic lives in `config/gradle/*.gradle` (`java.gradle`, `testing.gradle`, `static_analysis.gradle`, `version_updates.gradle`, `wrapper.gradle`, `packaging-ui.gradle`). Put new cross-module configuration there, not in module build files.
- All versions live in `gradle/libs.versions.toml`, including plugin versions. Never hard-code a version in a module build file or in `settings.gradle`.
- `gradle.properties` holds `group`, `version` and `java_version`.
- Detekt is configured in `config/detekt.yml`.
