# Storage, Catalog and Synchronization Instructions

Use this file when touching `dms-storage`: the file system, the catalog file, the library folders and their synchronization, scanning, or the `.dms-text/` store.

This module handles the user's real documents. A bug here loses data that may exist nowhere else. Correctness beats speed and elegance every time.

## Package layout

| Package | Purpose |
|---|---|
| `dms.storage` | `StorageAdapters`: builds every adapter for one app data directory. |
| `dms.storage.fs` | `FileSystemDocumentStorage` (file operations in one folder, name collisions), `PathResolver` (the one `RelativePathModel` ↔ `Path` mapping, NFC lookup), `AtomicFiles`, `ContentHasher`, `StorageIo` (I/O on `Dispatchers.IO`, exceptions to `StorageResult.Failure`). |
| `dms.storage.catalog` | `FileCatalogRepository`, `CatalogMapper`, `CatalogMigrations`, `CatalogFiles` (file name and conflict copies). DTOs in `dms.storage.catalog.dto`. |
| `dms.storage.scan` | `FileSystemLibraryScanner`, `ScanCache` (incremental hashing, kept in the app data directory), `IgnoredEntries`, `CloudPlaceholders`. DTOs in `dms.storage.scan.dto`, models in `dms.storage.scan.model`. |
| `dms.storage.text` | `FileTextStore` for `.dms-text/<documentId>.txt`. |
| `dms.storage.state` | Local state in the app data directory: `FileOperationJournal`, `FileTaskStore`, `FileActivityLog`, `FileSettingsRepository`, with their mappers. DTOs in `dms.storage.state.dto`. |
| `dms.storage.yaml` | `YamlMappers` (the one Jackson YAML configuration) and `YamlFile` (one atomically written YAML file). |
| `dms.storage.platform` | `SystemTrash` with `PlatformTrash` (system trash, `.dms-trash/` fallback) and `AppDataDirectory`. |

Tests use temporary folders, synthetic file names and a recording `SystemTrash`, never the system trash or a real library.

## File safety rules

- **Never overwrite** an existing user file. On a name collision, return a collision outcome or add a suffix (`name (2).pdf`) — the use case decides which.
- **Never delete** a user file except through the trash use case, and then move it to the trash (`java.awt.Desktop.moveToTrash`) instead of deleting permanently. Sync never deletes.
- **Write atomically**: write to a temporary file in the target directory (`.dms-tmp-<uuid>`), flush, then `Files.move(..., ATOMIC_MOVE)`. Temporary files start with `.dms-tmp-` so the scanner and the user can recognize and clean them up.
- **Copy verified**: hash the source, copy, hash the target, compare. Preserve the modification time.
- Use `java.nio.file` APIs only. No `java.io.File`.
- All I/O runs on `Dispatchers.IO` inside the adapter.

## Cross-platform paths

The app runs on macOS, Windows and Linux, and one library is often spread over file systems with different rules (APFS, NTFS, SMB network shares, ext4).

- Convert between `RelativePathModel` (NFC, `/`) and `java.nio.file.Path` in **one** mapper. When resolving a `RelativePathModel` in a folder, find the existing entry by NFC comparison, because macOS returns NFD names.
- Treat names case-insensitively when checking for collisions; never create two entries that differ only by case.
- Never build paths by string concatenation; use `Path.resolve`.

## The catalog file

- File name: `dms-catalog.yaml` in the root of every library folder. Format: YAML via Jackson 3 (`tools.jackson.*`), as in the rule-engine project.
- The file is **human-readable and stable**: documents sorted by path, categories and tags sorted by name, no volatile fields that change on every save. A diff between two versions must show only what changed.
- It starts with `formatVersion`. Any change to the stored structure:
  - adds fields as optional with a default, so older files still load, or
  - bumps `formatVersion` and adds a step to `CatalogMigrations`, with a test that reads the old fixture.
- Unknown fields are preserved, not dropped, so a newer version of the tool does not lose data when an older one saves on another device.
- The catalog stores **relative NFC paths only**. Folder locations are per-device settings and never go into the catalog.
- Never trust the catalog over the disk: the scanner compares both and reports differences; it does not "fix" either side.

## Library folders and synchronization

One folder is the **primary**, any number are **sync folders**. All are equal in content; the primary decides direction for "Sync to all destinations" and is written first.

- The **replication policy lives in the application layer** (`dms.application.sync.Replicator`), not here. Storage adapters work on **one folder per call** (`DocumentStorage`, `CatalogRepository`, `TextStore`) and persist the journal (`OperationJournal`); they never decide which folders to write.
- Order (enforced by `Replicator`): primary first, then each sync folder. If the primary fails, abort without touching the others. If a sync folder fails or is offline, the operation goes into the **failed-operation journal** (local app data directory, never inside a library folder) and the next folder continues.
- Every failure is a `StorageResult.Failure` with a `StorageFailure` reason, never an exception: an offline folder is `FOLDER_UNREACHABLE`, a missing source `NOT_FOUND`, an occupied target `ALREADY_EXISTS`. `RetryFailedOperations` relies on these reasons to drop operations that no longer apply.
- `copyFile` with `replaceExisting` moves the replaced file to the system trash before writing.
- All catalog copies carry the same `CatalogRevisionModel` after a successful write. On load, the higher revision wins; equal revisions with different content, and conflict copies, are merged by `CatalogMerger` per document id and reported.
- **Sync to all destinations**: copy what is missing or different in sync folders from the primary, apply catalog-known renames and moves. Files that exist only in a sync folder are reported, never removed.
- **Merge all folders**: scan every folder, copy each file missing anywhere into the folders that lack it. Same relative path with different content is a `ContentConflict` — never pick a winner automatically.
- Folders are compared by the use cases (`SyncFromPrimary`, `MergeAllFolders`, `FileComparison`) on the `FolderSnapshotModel`s the scanner returns: by NFC relative path, then content hash, then size when a hash is missing.

## Scanning

- A scan first lists the folder (cheap), then reads the files one by one and reports each to the `ScanListener`, so the UI can show "done of total". It checks for cancellation between two files and saves the `ScanCache` every 100 hashed files, so a cancelled first scan resumes where it stopped.
- `DocumentStorage.createdOn` returns a file's creation date in the system time zone, or `null` where the file system has none (the epoch counts as none). `DocumentDates` uses it only when the file name holds no date.

- A library folder is just a path, but any sync client may manage it. Files can then be **placeholders** that are not stored locally, and reading one triggers a download. Scans compare path, size and modification time first and hash only new or changed files. The DMS never calls a cloud vendor's API.
  - macOS: placeholders may be dataless files or `.<name>.icloud` stubs; treat stubs as "present, not downloaded" under their real name.
  - Windows: placeholders carry the `RECALL_ON_DATA_ACCESS` / `OFFLINE` attributes; do not read them during a scan.
- Sync clients create **conflict copies** (`dms-catalog 2.yaml`, `dms-catalog (conflicted copy).yaml`, …). Detect conflict copies of the catalog, merge them, and report them — never ignore them.
- Ignore: `.DS_Store`, `._*`, `Icon\r`, `Thumbs.db`, `desktop.ini`, `.dms-tmp-*`, `.dms-trash/`, `dms-catalog*.yaml` in the folder root, and `.dms-text/` (handled separately). The list lives in `IgnoredEntries` only.

## The `.dms-text/` store

- A hidden folder in every library folder: `.dms-text/<documentId>.txt`, UTF-8. Named by document id, so renames and moves do not touch it.
- It is derived data: written when a document is read for classification, replicated like any other write, and safe to rebuild.
- A missing or stale text file is never an error; search just finds less.
