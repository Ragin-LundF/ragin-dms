# ragin-dms

A desktop document management system that keeps documents as ordinary files and folders on disk.
Categories, tags and metadata live in a readable catalog file (`dms-catalog.yaml`) in the root of the
library. The library can be kept in one primary folder plus any number of sync folders, and a local AI
(LM Studio, Ollama or any OpenAI-compatible API) suggests titles, categories and tags for review.

Runs on macOS, Windows and Linux. Built with Kotlin and Compose Multiplatform.

## Modules

| Module | Purpose |
|---|---|
| `dms-domain` | Entities and domain rules. No dependencies. |
| `dms-application` | Use cases and ports. |
| `dms-storage` | File system, catalog file, synchronization of the library folders. |
| `dms-ai` | OpenAI-compatible LLM client, reading PDFs, images and Office files. |
| `dms-ui` | Compose Multiplatform desktop UI and composition root. |

Dependencies point inwards only: `dms-ui` → `dms-application` → `dms-domain`; the adapters
`dms-storage` and `dms-ai` implement ports of `dms-application`.

## Build

Requires JDK 25 (Gradle toolchains download it if missing).

* `./gradlew build` builds all modules and runs tests and Detekt.
* `./gradlew :dms-ui:run` starts the desktop app.
* `./gradlew :dms-ui:packageDmg` / `packageMsi` / `packageDeb` builds a native installer on the matching OS.
* `./gradlew versionCatalogUpdate` updates `gradle/libs.versions.toml` to the latest stable versions.

Shared build logic lives in `config/gradle/`, all versions in `gradle/libs.versions.toml`, Detekt rules
in `config/detekt.yml`.

## AI instructions

Instructions for AI coding assistants live in `.ai/` (entry points: `AGENTS.md`, `CLAUDE.md`).
