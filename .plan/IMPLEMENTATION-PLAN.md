# ragin-dms — Implementation Plan

Status: v2. It includes your answers to the open questions from 2026-10-21. The few points still open are in §11.

## 1. Goal

A desktop document management system for macOS, Windows and Linux that:

- keeps documents as **ordinary files and folders on disk**. The tool adds documents, creates folders and categorizes, but the library stays fully usable without the tool;
- stores categorization (category, tags, metadata) in a human-readable **catalog file** (`dms-catalog.yaml`) in the root of the library;
- keeps the library in **one primary folder plus any number of sync folders** (iCloud, Proton Drive, a NAS, an external disk, …). Each one holds the full library and its own copy of the catalog;
- detects changes in all folders and collects them on a **dashboard** as open tasks;
- suggests title, category, tags, document date, correspondent and file name with a **local AI** (LM Studio, Ollama or another OpenAI-compatible API). Everything is a suggestion that the user approves;
- has a clean, modern UI built with Compose Multiplatform;
- is written in Kotlin 2.4.21 with Clean Architecture.

## 2. What the sample library shows

From the structure of your copy in `~/IdeaProjects/tmp/Unterlagen`. Only names, counts and sizes were read, and no file content left your machine. The PDF text-layer check ran locally and reported counts only.

| Finding | Number | Consequence for the plan |
|---|---|---|
| Files / folders | 407 files, 71 folders, 244 MB (your full library has ~800) | Small: one catalog file and a full scan are cheap. |
| File types | 380 PDF, 15 DOCX, 3 PPTX, 1 PNG, plus single MP3, Audacity, SimpleMind and TXT files | PDF is the main case. Office text extraction is worth doing (§6). Unsupported files are catalogued and synced but not classified by the AI. |
| PDFs with a text layer | **370 of 390** (95 %); 20 are pure image scans | Most of your scans already went through OCR (scanner app). The **text layer is the main path** and the vision model handles the remaining 5 % (§6). |
| File names | 376 of 407 start with `YYYY-MM-DD` | Existing convention. Original names are kept, and AI rename suggestions follow this pattern. |
| Folder layout | Top level by area (authorities, house, people, children, telecoms), then by subject; files at depth 1–4 | Folders say *whose / which area*. Categories say *what kind of document* (invoice, payslip, insurance policy, tax). The two complement each other. |
| Umlauts in names | 91 names; **93 names are stored decomposed (NFD)** by macOS | All paths in the catalog are normalized to **NFC**, and comparisons between folders on different file systems (NAS, Windows, Linux) are NFC-based. Without this, `Ämter` on the Mac and on the NAS would look like two different folders. |
| Names with spaces | 353 | Fine on all platforms. Kept as they are. |
| System files | 3 `.DS_Store` | Ignored by scanner and sync. |

## 3. Key decisions

| Topic | Decision | Why |
|---|---|---|
| Language / runtime | Kotlin 2.4.21, Java 25 toolchain, Gradle 9.7.1 | As requested; same toolchain as rule-engine. |
| Build structure | Same as rule-engine (**Groovy**): `settings.gradle`, root `build.gradle` with plugins `apply false`, shared scripts in `config/gradle/*.gradle`, all versions in `gradle/libs.versions.toml`, Detekt in `config/detekt.yml`. The generated `buildSrc`, `app` and `utils` are removed. | Your choice; one build style across your projects. |
| Architecture | 5 modules: `dms-domain`, `dms-application`, `dms-storage`, `dms-ai`, `dms-ui` (§4), root package `dms` | Clean Architecture with the dependency rule enforced by Gradle module boundaries. |
| Platforms | macOS, Windows, Linux; Compose Multiplatform `jvm()` target; packages DMG, MSI, DEB (as rule-engine) built by a GitHub Actions matrix | Your answer. Each installer has to be built on its own OS. |
| Catalog | `dms-catalog.yaml` (visible) in every library folder; Jackson 3 YAML | Readable and diffable; ~800 documents ≈ 300 KB. |
| Library folders | One **primary** + any number of **sync folders**, chosen by the user. The tool writes every change to all of them. | Your answer to question 1 (§5). |
| Changes | Detected by scanning all folders, collected on the **dashboard** as tasks, applied only after user approval | Your answer to question 2 (§7). |
| AI integration | Own small OpenAI-compatible client (Ktor + kotlinx.serialization) behind a `DocumentClassifier` port. **Not Spring AI.** | Spring AI assumes a Spring Boot context; for a desktop app that is a lot of weight for one HTTP endpoint. Koog can replace the adapter later if agents are needed. |
| Reading documents | PDF text layer (PDFBox) → otherwise rendered pages to a vision model; images → vision model; DOCX/PPTX/XLSX → Apache POI text | §6. |
| Full-text search | Text of every document stored in the hidden `.dms-text/` folder | Your answer to question 11. |
| Secrets | API keys in the OS credential store (macOS Keychain, Windows Credential Manager, Linux Secret Service) via `java-keyring` | Works on all three platforms. |
| DI | Manual constructor injection in one composition root (`AppContainer`) | Enough for this size; Koin later if needed. |
| Architecture tests | Konsist in `dms-ui` `jvmTest` | Turns the `.ai` rules into failing tests. |

## 4. Modules

```text
ragin-dms/
├── config/
│   ├── detekt.yml
│   └── gradle/  java.gradle, testing.gradle, static_analysis.gradle,
│                version_updates.gradle, wrapper.gradle, packaging-ui.gradle
├── gradle/libs.versions.toml
├── settings.gradle
├── build.gradle
├── gradle.properties          group, version, java_version=25
├── dms-domain/                kotlin-jvm, no dependencies
├── dms-application/           kotlin-jvm, depends on dms-domain + coroutines
├── dms-storage/               kotlin-jvm, depends on dms-application + Jackson 3 YAML
├── dms-ai/                    kotlin-jvm, depends on dms-application + Ktor client + PDFBox + POI
└── dms-ui/                    KMP jvm(): commonMain → domain, application
                                          jvmMain   → + storage, ai, compose.desktop, java-keyring
```

### dms-domain (`dms.domain.*`)

- `document`: `Document(id, path, contentHash, size, kind, addedAt, metadata, categoryId, tags)`, `DocumentId`, `ContentHash`, `DocumentKind` (PDF, IMAGE, OFFICE, OTHER), `DocumentMetadata(title, documentDate, correspondent)`.
- `library`: `LibraryFolder(id, displayName, role = PRIMARY | SYNC)`, `LibraryFolderId`, `RelativePath` (always `/`-separated and NFC-normalized), `Folder`, `FileNamePolicy` (valid on macOS, Windows and Linux; no case-only collisions).
- `taxonomy`: `Category(id, name, parentId)`, `Tag`, `TagName` (normalized).
- `catalog`: `Catalog` aggregate (documents, categories, tags, revision) with pure functions `addDocument`, `moveDocument`, `renameDocument`, `assignCategory`, `addTags`, …; `CatalogRevision`; `CatalogMerger`.
- `classification`: `ClassificationSuggestion(title, category, tags, documentDate, correspondent, suggestedFileName, confidence)`, with each value marked as existing or new.
- `change`: what a scan found — `DetectedChange` sealed hierarchy: `NewFile`, `ModifiedFile`, `MovedFile`, `MissingFile`, `MissingInFolder`, `ContentConflict`, `CatalogConflict`.
- `task`: `OpenTask` sealed hierarchy for the dashboard: `ReviewSuggestion`, `ClassifyNewFile`, `ResolveConflict`, `RestoreOrDelete`, `SyncPending`, `RetryFailedOperation`.

### dms-application (`dms.application.*`)

Ports (`dms.application.port`):

| Port | Responsibility |
|---|---|
| `DocumentStorage` | per folder: import a file, create a folder, move/rename, move to trash, copy from another folder, list, open for reading |
| `CatalogRepository` | load and save the catalog of one folder |
| `LibraryScanner` | walk one folder and report its files (path, size, mtime, hash on demand) |
| `TextStore` | read and write `.dms-text/<documentId>.txt` |
| `DocumentContentReader` | document → `DocumentContent` (text or page images) |
| `DocumentClassifier` | content + existing taxonomy + naming convention → `ClassificationSuggestion` |
| `TaskStore` | pending suggestions and dismissed tasks (local app data) |
| `ActivityLog` | what the tool did, for the dashboard's history |
| `SettingsRepository` | library folders per device, AI provider, UI preferences |
| `Clock`, `IdGenerator` | testability |

Use cases (one class each, `suspend operator fun invoke`, sealed outcome):

- library: `ConfigureLibrary` (choose primary, add/remove sync folders), `OpenLibrary`, `ScanLibrary` (all folders → `DetectedChange`s), `BuildDashboard` (changes + suggestions → `OpenTask`s)
- sync: `SyncFromPrimary`, `MergeAllFolders` (no deletion), `RetryFailedOperations`
- folder: `CreateFolder`, `RenameFolder`, `MoveFolder`
- document: `AdoptNewFiles`, `AddDocuments`, `MoveDocument`, `RenameDocument`, `TrashDocument` (confirmation required, all folders), `RestoreDocument`, `UpdateMetadata`
- taxonomy: `AssignCategory`, `AddTags`, `RemoveTags`, `ManageCategories`, `RenameTag` / `MergeTags`
- classification: `SuggestClassification`, `ApproveSuggestion` (all or selected fields, including the rename), `RejectSuggestion`, `ClassifyAllNew` (batch, background)
- search: `SearchDocuments` (title, path, tags, category, date range, correspondent, and the text in `.dms-text/`)

### dms-storage (`dms.storage.*`)

- `fs`: atomic, verified file operations in one folder; NFC path mapping; case-collision checks.
- `catalog`: YAML DTOs, mapper, migrations, unknown-field preservation, stable sorting.
- `sync`: `ReplicatedDocumentStorage` (applies one operation to all folders), `ReplicatedCatalogRepository`, failed-operation journal (local app data), `FolderComparison`.
- `scan`: incremental scan (path/size/mtime first, hash only when needed).
- `text`: `.dms-text/` store.
- `cloud`: placeholder detection (iCloud/Proton on macOS, cloud-files attributes on Windows), conflict-copy detection, ignore list (`.DS_Store`, `Thumbs.db`, `desktop.ini`, `.dms-tmp-*`).

### dms-ai (`dms.ai.*`)

- `openai`: Ktor client for `/v1/chat/completions` (text and image content parts) and `/v1/models`; JSON-schema `response_format`.
- `classification`: prompt templates in resources, response validation, mapping to `ClassificationSuggestion`.
- `content`: PDFBox (text layer or rendered pages), images, Apache POI (DOCX/PPTX/XLSX).
- `config`: provider presets (LM Studio, Ollama, custom OpenAI-compatible).

### dms-ui (`dms.ui.*`)

Features: `dashboard`, `library`, `document` (inspector), `review` (suggestion approval), `sync`, `search`, `settings`, plus `components` and `theme`. Details in §8.

## 5. Library folders and synchronization

### Model

- The user picks **one primary folder** and adds any number of **sync folders**: iCloud, Proton Drive, a NAS share, an external disk, any directory. The primary can be changed later.
- Every folder holds the full library and its own `dms-catalog.yaml` with the same `libraryId`. Folder paths differ per device, so they are stored in the **app settings of each device**, not in the catalog.
- A folder that is not reachable (NAS offline, disk unplugged) is shown as offline. Operations for it go into the failed-operation journal and are retried when it is back.

### Operations done in the tool

Add, rename, move, create folder and trash are applied to **all reachable folders**, primary first. The catalog is saved with `revision + 1` to every folder. If the primary fails, nothing changes. If a sync folder fails, it is journaled and shown on the dashboard.

### Sync actions (started from the dashboard)

| Action | What it does | Deletes? |
|---|---|---|
| **Sync to all destinations** | Makes every sync folder contain what the primary contains: copies files that are missing or older there, and applies renames and moves the tool knows from the catalog. Files that exist only in a sync folder are reported, not removed. | Never |
| **Merge all folders** | Scans **all** folders, including the backups, and copies every file that is missing anywhere into the folders that lack it. Catalogs are merged per document id. | Never |
| **Resolve** (per task) | For a file missing in some folders: *restore* from a folder that has it, or *delete everywhere* (to the trash, after confirmation). For the same path with different content: *keep A*, *keep B*, or *keep both* (second one renamed). | Only on explicit confirmation |

Files are never overwritten silently. When the same path has different content, a `ContentConflict` task is created.

### Catalog consistency

- The higher `revision` wins. Equal revisions with different content, and conflict copies such as `dms-catalog 2.yaml` (several devices are in use), are merged by `CatalogMerger` per document id. The result is shown on the dashboard.
- **Moved files are recognized by content hash.** A file that disappeared from one path and appeared at another becomes a `MovedFile`, not "missing + new".
- Paths are compared NFC-normalized, so macOS (NFD) and NAS/Windows/Linux (NFC) folders match.
- Scans compare path, size and mtime first and hash only new or changed files, so a scan does not download cloud-only placeholders. With ~800 files a full scan of three folders takes seconds.

## 6. Reading documents and AI suggestions

### Reading (per document kind)

| Kind | How | Share in your library |
|---|---|---|
| PDF with text layer | PDFBox text, first pages up to a token budget | ~91 % |
| PDF without text layer | PDFBox renders the first 2 pages (~150 DPI, max 1600 px) → vision model | ~5 % |
| JPG / PNG | downscaled → vision model | < 1 % |
| DOCX / PPTX / XLSX | Apache POI text extraction (small effort, so included) | ~4 % |
| Everything else (MP3, project files, …) | not read; catalogued and synced, categorized manually | < 1 % |

A configurable option, **"Prefer vision for PDFs"**, sends rendered pages even when a text layer exists. Scanner OCR text can be poor, and the vision model then sees layout, logos and stamps as well. It is off by default because vision is slower.

The extracted text (text layer, POI text, or a transcription returned by the vision model) is written to `.dms-text/<documentId>.txt` in every library folder, so search works for scans too. The file is named by document id, so renames do not affect it.

### Suggestions

1. Build the prompt from: the document content (text or images), the existing categories and tags, the folder tree, **the naming convention** (`YYYY-MM-DD <Title>`, learned from the library), and today's date.
2. `POST {baseUrl}/chat/completions` with `response_format: json_schema` describing `ClassificationSuggestion` (title, category, tags, document date, correspondent, suggested file name, and a transcription when images were sent).
3. Validate the JSON. Invalid output becomes a typed failure with a retry option.
4. The suggestion becomes a **`ReviewSuggestion` task** on the dashboard. Nothing is written until the user approves it.
5. On approval, the user can accept all fields or selected ones and edit any value, including the file name. A rename is applied in **all** folders and in the catalog.

Default provider: LM Studio at `http://localhost:1234/v1`. Use a vision-capable model with good German and JSON support (e.g. Qwen2.5-VL / Qwen3-VL or Gemma 3) so one model covers text and images. The settings screen lists the loaded models and has a connection test. A remote provider is used only if configured explicitly.

Classification of many documents (the initial ~800) runs in the background with progress, pause and cancel.

## 7. The dashboard

The home screen. Everything that needs the user's attention is here, and nothing changes the library without an action taken here or in the library view.

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│ Dashboard                                        [Scan now] [Sync ▾] [Settings]│
├───────────────────────────────┬───────────────────────────────────────────────┤
│ Folders                       │ Open tasks (23)              [Approve selected]│
│  ★ iCloud      ✓ in sync      │  ☐ ✦ Review suggestion  2026-03-14 Kfz….pdf    │
│  ● Proton      ✓ in sync      │      Category: Versicherung · Tags: Auto, …    │
│  ● NAS         ⚠ offline      │      Rename → 2026-03-14 Kfz-Versicherung …    │
│                               │  ☐ ＋ 5 new files in Haus/Grundsteuer          │
│ Library                       │      [Adopt & suggest]                          │
│  812 documents                │  ☐ ⚠ Missing in Proton: 2 files  [Restore] [Del]│
│  37 uncategorized             │  ☐ ⇄ Same name, different content  [Resolve]    │
│  AI: LM Studio ✓ qwen3-vl     │  ☐ ↻ 3 operations failed on NAS   [Retry]       │
├───────────────────────────────┴───────────────────────────────────────────────┤
│ Recent activity: renamed 3 files · synced 12 files to Proton · scan 09:41      │
└───────────────────────────────────────────────────────────────────────────────┘
```

- **Open tasks** are grouped by type, can be filtered, and can be approved one by one or in bulk ("Approve all suggestions with high confidence" stays a manual click).
- Bulk actions: **Categorize all new**, **Sync to all destinations**, **Merge all folders**.
- Scans run on startup, on demand, and optionally every N minutes while the app is open. File watchers are not used as the source of truth because cloud folders report changes unreliably, but they can trigger a scan.
- The activity log is kept locally per device.

## 8. UI

- **Dashboard** (§7): home screen.
- **Library**: sidebar with the folder tree, categories and tags; document list (name, category, tags, date, correspondent); multi-select; drag and drop from the file manager to add, and inside the tree to move.
- **Inspector**: preview (PDFBox renders the page; images directly; an icon for Office and other files), metadata, tag chips with autocomplete, the pending suggestion of this document, "show in Finder/Explorer".
- **Review**: a side-by-side view for a suggestion (preview on the left, fields on the right, with new values highlighted), editable before approval, with keyboard flow (approve → next).
- **Sync**: folder cards (status, last scan, differences), sync actions, and conflicts with a comparison.
- **Settings**: library folders (primary and sync folders, per device), AI provider, theme.
- Material 3, light and dark theme, keyboard shortcuts, native menu bar on macOS.

## 9. Phases

Every phase ends with `./gradlew build` green (tests + Detekt) and your review before the next one starts.

| Phase | Content | Result you can see |
|---|---|---|
| **0. Build setup** | rule-engine structure (Groovy, `config/gradle/*.gradle`, Detekt, `.editorconfig`), version catalog (Kotlin 2.4.21, Compose MP, coroutines, Jackson 3, Ktor, PDFBox, POI, java-keyring, Konsist), five empty modules, GitHub Actions build for macOS/Windows/Linux. | `./gradlew build` is green; an empty window opens. |
| **1. Domain** | Domain types, `RelativePath` (NFC), `FileNamePolicy` (cross-platform), `Catalog`, `CatalogMerger`, change and task types, tests. | Tested core model. |
| **2. Application** | Ports, use cases, in-memory fakes, tests (including sync and merge scenarios on fake folders). | All behavior testable without disk or UI. |
| **3. Storage** | File operations, catalog YAML, scanner, `.dms-text` store, replicated storage over N folders, journal, placeholder and conflict-copy detection. Tests on temp dirs with NFD and NFC names. | Library read and written on disk in several folders. |
| **4. UI: dashboard + library** | Composition root, theme, settings (folders), dashboard with scan results and sync actions, library view, inspector, manual categorization and tagging, rename. | Usable DMS without AI, syncing to all folders. |
| **5. AI** | OpenAI-compatible client (text and images), PDFBox/POI readers, prompts, `ReviewSuggestion` tasks, review screen, batch classification in the background, AI settings. | Suggestions for all your documents, approved on the dashboard. |
| **6. Search** | Search over the catalog and `.dms-text/`, filters, saved searches. | Find documents by words inside scans. |
| **7. Polish & packaging** | Preview quality, keyboard flow, Konsist architecture tests, installers (DMG/MSI/DEB) with icons, import of your existing ~800 files as the first real run (on a copy first). | Installable app on all three platforms. |

The order changed from v1: syncing is now part of storage (phase 3) because every operation writes to all folders from the start, and the dashboard comes with the first UI.

## 10. Risks

| Risk | Mitigation |
|---|---|
| Data loss in your documents | No overwrite, no delete without confirmation (and only to the trash), atomic and hash-verified copies, all tests on temp dirs, first real run on a copy. |
| A sync folder diverges (offline NAS, edits on another device) | Scans of all folders, journal and retry, merge without deletion, conflicts as tasks. |
| Unicode / case differences between file systems | NFC everywhere in the catalog; `FileNamePolicy` blocks case-only collisions and characters Windows rejects. |
| Catalog conflicts from several devices | Revision counter, per-document merge, conflict-copy detection. |
| Scans downloading the whole cloud library | Metadata-first scanning, lazy hashing. |
| Local model quality / speed | Everything is reviewed; prompts in resource files; text layer first; vision only where needed; background processing. |
| Compose MP and Kotlin 2.4.21 compatibility | Pin matching versions in phase 0; carry over rule-engine's coroutines workaround. |
| Installers per OS | GitHub Actions matrix, as in rule-engine's `build-desktop.yml`. |

## 11. Still open

1. **Primary folder on a device without it.** On Linux there is no iCloud client. Should a device be allowed to use a sync folder (e.g. Proton or the NAS) as its primary? (**Assumed yes:** the primary is a setting per device.)
2. **Deletions made outside the tool.** If you delete a file in Finder in the primary folder, should the dashboard offer "delete everywhere" (to trash) as the default action, or "restore from a sync folder"? (**Assumed: restore**, because it is safer.)
3. **Your library has ~800 files, the copy 407.** Is the copy a representative subset? Nothing in the plan depends on it.
