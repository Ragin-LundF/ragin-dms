# Storage, Catalog and Synchronization Instructions

Use this file when touching `crates/infra-storage` (`dms_infra_storage`): the file system, the catalog file, the library folders and their synchronization, scanning, the `.dms-text/` store, or the local state in the app data directory.

This crate handles the user's real documents. A bug here loses data that may exist nowhere else. Correctness beats speed and elegance every time.

## Module layout

| Module | Purpose |
|---|---|
| `storage_adapters` | `StorageAdapters`: builds every adapter for one app data directory (`new`, or `with_trash` for tests). |
| `fs` | `FileSystemDocumentStorage` (file operations in one folder, name collisions), `path_resolver` (the one `RelativePath` ↔ `Path` mapping, NFC lookup), `atomic_files` (atomic writes, verified copies, placing without replacing), `content_hasher`, `folder_walk`, `storage_io` (I/O errors to `StorageError`). |
| `catalog` | `FileCatalogRepository` (refuses to save over a newer revision: `CatalogChanged`), `CatalogMigrations`, `CatalogFiles` (file name and conflict copies). Mappers (`catalog_mapper`, `settings_mapper`, `field_mapper`, `query_mapper`, `rule_mapper`, and `unknown_fields`, which carries fields of newer versions over) in `catalog::mapper`. DTOs in `catalog::dto`; every DTO with an identity or a singleton implements `ExtensibleDto`. |
| `yaml` | The YAML layer of every file the tool writes: `saphyr-parser` events into an own `Node` tree with Jackson's JSON schema, and an own port of the snakeyaml-engine emitter with Jackson's quoting rules, so the output is byte-identical to version 1.x. `YamlFile`: one atomically written YAML file. No serde. |
| `scan` | `FileSystemLibraryScanner`, `ScanCache` (incremental hashing, kept in the app data directory), `IgnoredEntries`, `CloudPlaceholders`. |
| `text` | `FileTextStore` for `.dms-text/<documentId>.txt`. |
| `state` | Local state in the app data directory: `FileOperationJournal`, `FileTaskStore`, `FileActivityLog`, `FileSettingsRepository`, with their mappers. DTOs in `state::dto`. |
| `export` | `FileExportWriter` (copies an export bundle into a folder or a zip outside the library) and `index_files` (the CSV and HTML index). |
| `index` | `FileVectorIndex`: the embedding vectors per document for Ask (`.vec`, Java `DataOutput` format), in a hidden folder of every library folder. |
| `plan` | `FilePlanStore`: the restructure records in `.dms-plans/`, replicated, so a restructure can be reversed. |
| `watch` | `FileSystemFolderWatcher` (`notify`): tells the app that a library folder changed on disk, so it can scan. |
| `platform` | `SystemTrash` with `PlatformTrash` (the `trash` crate, `.dms-trash/` fallback) and `AppDataDirectory`. |

Tests use temporary folders, synthetic file names and a recording `SystemTrash`, never the system trash or a real library.

## File safety rules

- **Never overwrite** an existing user file. On a name collision, return a collision outcome or add a suffix (`name (2).pdf`) — the use case decides which.
- **Never delete** a user file except through the trash use case, and then move it to the trash (`SystemTrash`) instead of deleting permanently. Sync never deletes. No `fs::remove_file`/`remove_dir_all` on user files.
- **Write atomically**: write to a temporary file in the target directory (`.dms-tmp-<uuid>`), flush and sync, then move it into place. Temporary files start with `dms_domain::library::TemporaryNames::PREFIX` (`.dms-tmp-`) so the scanner and the user can recognize and clean them up.
- Replace by rename only where replacing is intended (the catalog and state files). A new file is placed without replacing (`atomic_files::move_without_replacing`: `renameat2`/`renamex_np` with `NOREPLACE` via `rustix` on macOS and Linux, check plus rename on Windows), because a plain `rename` replaces an existing target; check for a case-insensitive clash right before the move.
- **Copy verified**: hash the source, copy, hash the target, compare. Preserve the modification time.
- Every failure is returned as `StorageResult`, never a panic; an `io::Error` becomes `StorageFailure::IoError` with its message as detail.

## Cross-platform paths

The app runs on macOS, Windows and Linux, and one library is often spread over file systems with different rules (APFS, NTFS, SMB network shares, ext4).

- Convert between `RelativePath` (NFC, `/`) and `std::path::Path` in **one** mapper (`fs::path_resolver`). When resolving a `RelativePath` in a folder, find the existing entry by NFC comparison, because macOS returns NFD names.
- Treat names case-insensitively when checking for collisions; never create two entries that differ only by case.
- Never build paths by string concatenation; use `Path::join` segment by segment through the resolver.

## The catalog file

- File name: `dms-catalog.yaml` in the root of every library folder. Format: YAML exactly as version 1.x wrote it with Jackson 3; the `yaml` module reproduces Jackson's output byte for byte (quoting, wrapping at column 80, key order, `null`s). Do not "fix" a Jackson quirk: 1.x must still read the file, and an unchanged catalog must not change on save. The golden files in `tests/fixtures/jackson/` prove the parity.
- The file is **human-readable and stable**: documents sorted by path, categories and tags sorted by name (UTF-16 order, as 1.x), no volatile fields that change on every save. A diff between two versions must show only what changed.
- It starts with `formatVersion`. Any change to the stored structure:
  - adds fields as optional with a default, so older files (and version 1.x) still load, or
  - bumps `formatVersion` and adds a step to `CatalogMigrations`, with a test that reads the old fixture.
- Unknown fields are preserved, not dropped, and keep their order, so a newer version of the tool does not lose data when an older one saves on another device.
- The catalog stores **relative NFC paths only**. Folder locations are per-device settings and never go into the catalog.
- Never trust the catalog over the disk: the scanner compares both and reports differences; it does not "fix" either side.
- The same compatibility rule applies to every other file this crate writes (settings, tasks, activity, journal, scan cache, plans, `.vec`, export index): version 1.x and 2.x share them, with no migration.

## Library folders and synchronization

One folder is the **primary**, any number are **sync folders**. All are equal in content; the primary decides direction for "Sync to all destinations" and is written first.

- The **replication policy lives in the application layer** (`dms_application::sync::Replicator`), not here. Storage adapters work on **one folder per call** (`DocumentStorage`, `CatalogRepository`, `TextStore`) and persist the journal (`OperationJournal`); they never decide which folders to write.
- Order (enforced by `Replicator`): primary first, then each sync folder. If the primary fails, abort without touching the others. If a sync folder fails or is offline, the operation goes into the **failed-operation journal** (local app data directory, never inside a library folder) and the next folder continues.
- Every failure is a `StorageError` with a `StorageFailure` reason, never a panic: an offline folder is `FolderUnreachable`, a missing source `NotFound`, an occupied target `AlreadyExists`. `RetryFailedOperations` relies on these reasons to drop operations that no longer apply.
- `copy_file` with `replace_existing` moves the replaced file to the system trash before writing.
- All catalog copies carry the same `CatalogRevision` after a successful write. On load, the higher revision wins; equal revisions with different content, and conflict copies, are merged by `CatalogMerger` per document id and reported.
- **Sync to all destinations**: copy what is missing or different in sync folders from the primary, apply catalog-known renames and moves. Files that exist only in a sync folder are reported, never removed.
- **Merge all folders**: scan every folder, copy each file missing anywhere into the folders that lack it. Same relative path with different content is a `ContentConflict` — never pick a winner automatically.
- Folders are compared by the use cases (`SyncFromPrimary`, `MergeAllFolders`, `FileComparison`) on the `FolderSnapshot`s the scanner returns: by NFC relative path, then content hash, then size when a hash is missing.

## Scanning

- A scan first lists the folder (cheap), then reads the files one by one and reports each to the `ScanListener`, so the UI can show "done of total". It checks `ScanListener::is_cancelled` between two files (a cancelled scan ends with `StorageFailure::Cancelled`) and saves the `ScanCache` every 100 hashed files, so a cancelled first scan resumes where it stopped.
- `DocumentStorage::created_on` returns a file's creation date in the system time zone, or `None` where the file system has none (the epoch counts as none). `DocumentDates` uses it only when the file name holds no date.
- A library folder is just a path, but any sync client may manage it. Files can then be **placeholders** that are not stored locally, and reading one triggers a download. Scans compare path, size and modification time first and hash only new or changed files. The DMS never calls a cloud vendor's API.
  - macOS: placeholders may be dataless files or `.<name>.icloud` stubs; treat stubs as "present, not downloaded" under their real name.
  - Windows: placeholders carry the `RECALL_ON_DATA_ACCESS` / `OFFLINE` attributes; do not read them during a scan.
- Sync clients create **conflict copies** (`dms-catalog 2.yaml`, `dms-catalog (conflicted copy).yaml`, …). Detect conflict copies of the catalog, merge them, and report them — never ignore them.
- Ignore: `.DS_Store`, `._*`, `Icon\r`, `Thumbs.db`, `desktop.ini`, `.dms-tmp-*`, `.dms-trash/`, `dms-catalog*.yaml` in the folder root, and `.dms-text/` (handled separately). The list lives in `IgnoredEntries` only.

## The `.dms-text/` store

- A hidden folder in every library folder: `.dms-text/<documentId>.txt`, UTF-8. Named by document id, so renames and moves do not touch it.
- It is derived data: written when a document is read for classification, replicated like any other write, and safe to rebuild.
- A missing or stale text file is never an error; search just finds less.
