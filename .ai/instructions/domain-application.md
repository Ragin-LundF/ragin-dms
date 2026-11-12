# Domain and Use Case Instructions

Use this file when touching `crates/domain` (`dms_domain`) or `crates/application` (`dms_application`).

## Module layout

| Module | Purpose |
|---|---|
| `dms_domain::document` | `Document`, `DocumentId`, `DocumentKind`, `DocumentMetadata` (title, document date, correspondent), `ContentHash`. |
| `dms_domain::library` | `LibraryFolder`, `LibraryFolderId`, `LibraryFolders`, `FolderRole` (`Primary`, `Sync`), `RelativePath`, `FileNamePolicy`, `TemporaryNames`. |
| `dms_domain::taxonomy` | `Category`, `CategoryId`, `TagName`. |
| `dms_domain::catalog` | `Catalog` (the aggregate root), `CatalogRevision`, `CatalogUpdate`, `CatalogMerger`, `CatalogRejection`, and `CatalogRecords` (contracts, cases and tax years, set with `Catalog::with_records`). |
| `dms_domain::classification` | `ClassificationSuggestion` and its parts. Suggestions are values, never applied state. |
| `dms_domain::change` | `DetectedChange` — what a scan found (`NewFile`, `ModifiedFile`, `MovedFile`, `MissingFile`, `ContentConflict`) — and `ChangeDetector`, which compares the catalog with `FolderSnapshot`s. |
| `dms_domain::task` | `OpenTask`: what the dashboard asks the user to decide, with a stable `TaskKey`. `OpenTask::for_change` maps every `DetectedChange` to its task. |
| `dms_domain::field`, `retention`, `deadline` | Custom fields (`FieldDefinition`, `FieldValue`, `FieldValues` parsing, `Decimal` with `BigDecimal`-like equality), the strictly opt-in retention (`RetentionDates`), and `DeadlinePlanner` for the deadline and retention tasks. |
| `dms_domain::query`, `settings`, `link` | `DocumentQuery` (the one filter for library, smart folders, restructure, export and Ask; `tax: Option<TaxFilter>`), `SavedSearch`, the library-wide `CatalogSettings` with `LibraryLayout` and `InboxSettings` (also `propose_splits`, `convert_emails`, `keep_email_originals`), and `DocumentLink`. 2.1 added to `CatalogSettings`: `owner`, `bills_tracked_since`, `dismissed_contract_proposals`, `expectations`, `emergency_folder`. |
| `dms_domain::money`, `bill` | `Money`, `MoneyTotals`, `BillingPeriod`; bills: `BillStatus`, `BillPlanner`, `EpcPayment` (payment QR data), `EInvoice`, `Iban`. |
| `dms_domain::contract`, `cases`, `tax` | `Contract`, `ContractTerms`, `Notice`, `ContractPlanner`, `ContractProposer`; `Case`, `CaseEntry`, `FollowUp`, `CasePlanner`; `TaxYear`, `TaxArea`, `TaxAssignment`, `TaxFilter`, `TaxChecklistBuilder`, `TaxSums`. |
| `dms_domain::expectation`, `emergency`, `health`, `split` | `ExpectedDocument`, `ExpectationOverride`, `ExpectationSnoozes`, `RhythmDetector`, `ExpectationPlanner`; `EmergencyFolder`, `EmergencyItem`, `EmergencyPlanner`; `HealthFinding`, `HealthSchedule`, `HealthJudge`, `SnapshotDiff`, `SnapshotRetention`; `SplitProposal`, `SplitPart`, `SplitProblem`. |
| `dms_domain::rule`, `restructure`, `outline`, `rebuild` | Matching rules (`RuleMatcher` in `rule::matching`, `RuleLearner`), `PathTemplate`, `RestructurePlanner`, `FolderShape`, the `FolderOutline` the AI gets for a target folder, and the layout types of "Convert to real PDF". |
| `dms_domain::text` | The string semantics of the 1.x rules: whitespace and `trim` as Kotlin defined them, lengths in UTF-16 units, UTF-16 sort order. Use these helpers wherever a rule trims, measures or sorts text. |
| `dms_application::port` | Every port trait, grouped by area (`storage`, `state`, `ai`, `assistant`, `pdf`, `mail`, `export`, `index`, `plan`, `sidecar`, `health`, `system`). |
| `dms_application::common` | Shared outcome (`OperationOutcome`), `Progress`, `SiblingNames` (name collision checks against the primary and the catalog), `ActivityRecorder`, `DocumentDates`, and `source_texts` (the texts of several documents for the AI, cut to one budget). |
| `dms_application::<feature>` | Use cases grouped by feature: `library` (with `CatalogStore`), `sync` (with `Replicator`), `folder`, `document`, `catalog`, `resolution`, `classification`, `search` (`ContentMatcher` is the shared text index the searching use cases use; `TranslateQuery` turns plain words into filters with the AI), `inbox`, `restructure`, `deadline`, `rule`, `export`, `pdf`, `ask`, `assistant`, `preview` (thumbnails, stored per content hash through `PreviewStore`), `sidecar` (`MigrateSidecarLayout`, which moves the flat 1.x `.dms-text/`/`.dms-index/` files into subfolders). New in 2.1: `bill`, `expectation`, `contract`, `tax`, `cases`, `emergency`, `letter` (`DraftLetter`, `SaveLetter`), `mail` (`ImportEmails`), `health` (`VerifyLibrary`, `CatalogSnapshotKeeper`, `RestoreCatalogSnapshot`, `CompareCatalogSnapshot`, `RepairCopy`), `split` (`ProposeSplit`, `ApplySplit`). Inputs and outcomes go in `<feature>::model`. |
| `dms_application::services` | `AppPorts` (every adapter the app needs) and `AppServices::new(ports)`, which builds every use case once. |

The data types of every module above live in its `model` submodule (`dms_domain::document::model::Document`) and are re-exported one level up; see "Models and DTOs" in `coding-guidelines.md`.

## Domain rules

- `Catalog` is the aggregate root. Documents, categories and tags are changed **through** `Catalog` functions that return a new catalog (`CatalogUpdate`, or `Result<CatalogUpdate, DomainError>` where the input can be invalid); nothing mutates a document list from outside. `Catalog::new` validates the whole aggregate.
- Contracts, cases and tax years are `CatalogRecords` beside the documents; `CatalogMerger` merges them per id, the newer copy winning without a reported conflict.
- `DeadlinePlanner` skips the due date of a paid bill, and of an overdue one (`BillPlanner` raises `BillOverdue` instead), and a cancel-by field of a document whose contract plans its own cancel-by date. `BuildDashboard` adds the tasks of the per-feature planners (`BillPlanner`, `ExpectationPlanner`, `ContractPlanner`, `CasePlanner`, `EmergencyPlanner`).
- Every catalog change increments `CatalogRevision`. The revision decides which of two catalog copies is newer — never the file modification time, which cloud sync rewrites.
- A document is identified by `DocumentId` (a UUID), not by its path. Paths change when the user moves or renames a file; the id does not.
- `ContentHash` (SHA-256) is how a moved or renamed file is recognized again. Keep it in the catalog.
- `RelativePath` is always relative to a library folder, uses `/` as separator, is **NFC-normalized** in its constructor, and is never absolute. macOS may store umlauts decomposed (NFD), Windows, Linux and network shares composed (NFC); without normalization the same file looks like two.
- `FileNamePolicy` is the only validator for file and folder names from the user or the AI. A name must be valid on macOS, Windows **and** Linux: no `<>:"/\|?*` or control characters, no reserved Windows names (`CON`, `NUL`, …), no trailing dots or spaces, length limit (UTF-16 units), and no name that differs from a sibling only by case. Do not validate names anywhere else.
- Keep original file names. A new name only comes from an approved suggestion or a user rename.
- Tag names are normalized once, in `TagName`'s constructor (trimmed, case-insensitive uniqueness, display casing preserved). Never compare raw tag strings.
- Categories form a tree (`parent_id`). A document has at most one category and any number of tags. Folders say *where / whose*, categories say *what kind of document* — do not derive one from the other.
- A document of `DocumentKind::Other` is catalogued and synced but never sent to the AI.
- Validation returns `DomainError::InvalidArgument` with the 1.x message; it never panics on user input.

## Use case rules

- One use case per struct, named as a verb phrase (`AddDocuments`, `SyncFromPrimary`, `ApproveSuggestion`), holding `Arc`s of its ports and collaborators, with `pub fn new(..)` and exactly one `pub fn run(&self, ..)`.
- A use case returns an outcome enum (usually `OperationOutcome`). The desktop app decides how to present each variant; the use case does not produce user-facing text.
- Use cases are stateless. State lives in the catalog (persisted, shared), in the task store and activity log (local), or in the desktop app's session (transient). The few use cases with caches are built once in `AppServices`.
- **Nothing changes the library without a user action.** Scans only detect; they produce `DetectedChange`s, and `BuildDashboard` turns them into `OpenTask`s. Adopting, renaming, syncing, restoring and deleting are separate use cases the user triggers. The one documented exception is the inbox: when the user switched it on (opt-in), `CollectInbox` catalogs the new files a scan found in it.
- `CatalogStore` is the only writer of the catalog. A save based on an older revision than the primary holds is refused (`StorageFailure::CatalogChanged`); the store then merges the primary's catalog, applies the edit again and saves once more. `Replicator` never lowers a sync folder's revision. After every successful save it keeps a catalog snapshot (`CatalogSnapshotKeeper`) in the app data directory.
- `TaskStore` holds dismissed tasks and snoozes (`snooze`, `snoozes`) of expected documents.
- A use case that writes files writes the **file first and the catalog second**. A crash in between leaves an uncatalogued file, which the next scan finds; the reverse would leave a catalog entry pointing at nothing.
- Every write goes to **all reachable library folders**, primary first. A failed primary aborts the operation; a failed sync folder is journaled and becomes a `RetryFailedOperations` task. Use `Replicator` for this; never loop over folders in a use case.
- Sync never deletes. Deleting is its own use case (`TrashDocument`), requires `confirmed: bool` in its input (without it: `OperationOutcome::NotConfirmed`, nothing touched), and moves to the trash in every folder.
- Duplicates are files with the same size and SHA-256 that the storage also finds identical byte by byte (`DocumentStorage::same_content`) right before anything moves; a name or a size alone never makes a duplicate. `MergeDuplicates` keeps one catalogued copy, takes over the assigned values of the others and trashes them per folder only where that folder's comparison passed; `TrashDuplicateFile` does the same for a new file that copies a document. Both need the confirmation flag.
- A file found under another name in one folder (`MovedFile`) is brought in line by renaming it there (`MoveDecision::Undo`), never by copying, which would leave a duplicate.
- `search::SearchContents` finds documents by their text: the AI's transcription from the `TextStore` plus the text layer from `DocumentContentReader::text`, which never renders pages and so needs no AI server. It keeps the texts in memory by content hash and never writes.
- AI is optional. Every use case except the classification and assistant ones (`dms_application::assistant`: summary, tags, questions; and `ExtractContractTerms`, `ProposeSplit`, `TranslateQuery`) must work when no AI server is configured. `DraftLetter` falls back to its fixed template without AI.
- Long use cases take `&dyn Progress<P>`, report each step and check `is_cancelled()` between items; only stop where the library stays consistent.

## Extending the catalog model

Adding a field to `Document` or `Catalog` touches, in order:

1. The domain type in `crates/domain`, and `Catalog::new`'s validation if the field has rules.
2. The catalog DTO and mapper in `crates/infra-storage` (`catalog::dto`, `catalog::mapper`), plus a format version decision — see `infra-storage.md`. Check the golden files still match.
3. `CatalogMerger`, or the field is lost when copies from two folders or devices are merged.
4. `ClassificationSuggestion`, the JSON schema and the prompt in `crates/infra-ai`, if the AI should fill it.
5. The DTOs, presenters and screens in `src-tauri` and `ui` (quick look, review).

## Adding a detected change or a task type

1. The `DetectedChange` or `OpenTask` variant.
2. The detection in `ChangeDetector`, with a test on synthetic snapshots.
3. The mapping in `OpenTask::for_change` (exhaustive `match`, so the compiler names it).
4. The use case that resolves it, and the dashboard row with its actions in `src-tauri/src/features/dashboard`. The `match` over `OpenTask` there is exhaustive, so the compiler names what is missing.
