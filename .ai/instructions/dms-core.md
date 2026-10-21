# Domain and Use Case Instructions

Use this file when touching `dms-domain` or `dms-application`.

## Package layout

| Package | Purpose |
|---|---|
| `dms.domain.document` | `Document`, `DocumentId`, `DocumentKind`, `DocumentMetadata` (title, document date, correspondent), `ContentHash`. |
| `dms.domain.library` | `LibraryFolder`, `LibraryFolderId`, `FolderRole` (`PRIMARY`, `SYNC`), `RelativePath`, `Folder`, `FileNamePolicy`. |
| `dms.domain.taxonomy` | `Category`, `CategoryId`, `Tag`, `TagName`. |
| `dms.domain.catalog` | `Catalog` (the aggregate root), `CatalogRevision`, `CatalogMerger`. |
| `dms.domain.classification` | `ClassificationSuggestion` and its parts. Suggestions are values, never applied state. |
| `dms.domain.change` | `DetectedChange` sealed hierarchy: what a scan found (`NewFile`, `ModifiedFile`, `MovedFile`, `MissingFile`, `MissingInFolder`, `ContentConflict`, `CatalogConflict`). |
| `dms.domain.task` | `OpenTask` sealed hierarchy: what the dashboard asks the user to decide. |
| `dms.application.port` | Every port interface. |
| `dms.application.<feature>` | Use cases grouped by feature: `library`, `sync`, `folder`, `document`, `taxonomy`, `classification`, `search`. Inputs and outcomes go in `<feature>.model`. |

## Domain rules

- `Catalog` is the aggregate root. Documents, categories and tags are changed **through** `Catalog` functions that return a new `Catalog`; nothing mutates a document list from outside.
- Every catalog change increments `CatalogRevision`. The revision decides which of two catalog copies is newer — never the file modification time, which cloud sync rewrites.
- A document is identified by `DocumentId` (a UUID), not by its path. Paths change when the user moves or renames a file; the id does not.
- `ContentHash` (SHA-256) is how a moved or renamed file is recognized again. Keep it in the catalog.
- `RelativePath` is always relative to a library folder, uses `/` as separator, is **NFC-normalized** in its factory, and is never absolute. macOS may store umlauts decomposed (NFD), Windows, Linux and network shares composed (NFC); without normalization the same file looks like two.
- `FileNamePolicy` is the only validator for file and folder names from the user or the AI. A name must be valid on macOS, Windows **and** Linux: no `<>:"/\|?*` or control characters, no reserved Windows names (`CON`, `NUL`, …), no trailing dots or spaces, length limit, and no name that differs from a sibling only by case. Do not validate names anywhere else.
- Keep original file names. A new name only comes from an approved suggestion or a user rename.
- Tag names are normalized once, in `TagName`'s factory (trimmed, case-insensitive uniqueness, display casing preserved). Never compare raw tag strings.
- Categories form a tree (`parentId`). A document has at most one category and any number of tags. Folders say *where / whose*, categories say *what kind of document* — do not derive one from the other.
- A document of `DocumentKind.OTHER` is catalogued and synced but never sent to the AI.

## Use case rules

- One use case per class, named as a verb phrase (`AddDocuments`, `SyncFromPrimary`, `ApproveSuggestion`), with a single `suspend operator fun invoke(...)`.
- A use case returns a `sealed interface` outcome. The UI decides how to present each case; the use case does not produce user-facing text.
- Use cases are stateless. State lives in the catalog (persisted, shared), in the task store and activity log (local), or in the UI's view models (transient).
- **Nothing changes the library without a user action.** Scans only detect; they produce `DetectedChange`s, and `BuildDashboard` turns them into `OpenTask`s. Adopting, renaming, syncing, restoring and deleting are separate use cases the user triggers.
- A use case that writes files writes the **file first and the catalog second**. A crash in between leaves an uncatalogued file, which the next scan finds; the reverse would leave a catalog entry pointing at nothing.
- Every write goes to **all reachable library folders**, primary first. A failed primary aborts the operation; a failed sync folder is journaled and becomes a `RetryFailedOperation` task.
- Sync never deletes. Deleting is its own use case (`TrashDocument`), requires an explicit confirmation flag in its input, and moves to the trash in every folder.
- AI is optional. Every use case except the classification ones must work when no `DocumentClassifier` is configured.

## Extending the catalog model

Adding a field to `Document` or `Catalog` touches, in order:

1. The domain type in `dms-domain`.
2. The catalog DTO and mapper in `dms-storage` (`dms.storage.catalog`), plus a format version decision — see `dms-storage.md`.
3. `CatalogMerger`, or the field is lost when copies from two folders or devices are merged.
4. `ClassificationSuggestion`, the JSON schema and the prompt in `dms-ai`, if the AI should fill it.
5. The inspector, the review screen and their view models in `dms-ui`.

## Adding a detected change or a task type

1. The `DetectedChange` or `OpenTask` subtype (same package as its sealed parent).
2. The detection in `ScanLibrary` / the folder comparison, with a test on fake folders.
3. The mapping in `BuildDashboard`.
4. The use case that resolves it, and the dashboard row with its actions. The `when` over `OpenTask` in the UI is exhaustive, so the compiler names what is missing.
