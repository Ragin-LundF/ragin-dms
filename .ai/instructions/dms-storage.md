# Storage, Catalog and Synchronization Instructions

Use this file when touching `dms-storage`: the file system, the catalog file, the library folders and their synchronization, scanning, or the `.dms-text/` store.

This module handles the user's real documents. A bug here loses data that may exist nowhere else. Correctness beats speed and elegance every time.

## Package layout

| Package | Purpose |
|---|---|
| `dms.storage.fs` | `FileSystemDocumentStorage`: file operations in one folder, atomic writes, NFC path mapping, name collision handling. |
| `dms.storage.catalog` | `FileCatalogRepository`, `CatalogMapper`, `CatalogMigrations`. DTOs in `dms.storage.catalog.dto`. |
| `dms.storage.sync` | `ReplicatedDocumentStorage`, `ReplicatedCatalogRepository`, the failed-operation journal, `FolderComparison`. |
| `dms.storage.scan` | Walking one folder, incremental hashing, producing the raw facts the use cases turn into `DetectedChange`s. |
| `dms.storage.text` | `FileTextStore` for `.dms-text/<documentId>.txt`. |
| `dms.storage.cloud` | Cloud and platform specifics: placeholders (not downloaded files), conflict copies, the ignore list. |

## File safety rules

- **Never overwrite** an existing user file. On a name collision, return a collision outcome or add a suffix (`name (2).pdf`) — the use case decides which.
- **Never delete** a user file except through the trash use case, and then move it to the trash (`java.awt.Desktop.moveToTrash`) instead of deleting permanently. Sync never deletes.
- **Write atomically**: write to a temporary file in the target directory (`.dms-tmp-<uuid>`), flush, then `Files.move(..., ATOMIC_MOVE)`. Temporary files start with `.dms-tmp-` so the scanner and the user can recognize and clean them up.
- **Copy verified**: hash the source, copy, hash the target, compare. Preserve the modification time.
- Use `java.nio.file` APIs only. No `java.io.File`.
- All I/O runs on `Dispatchers.IO` inside the adapter.

## Cross-platform paths

The app runs on macOS, Windows and Linux, and one library is often spread over file systems with different rules (APFS, NTFS, SMB/NAS, ext4).

- Convert between `RelativePath` (NFC, `/`) and `java.nio.file.Path` in **one** mapper. When resolving a `RelativePath` in a folder, find the existing entry by NFC comparison, because macOS returns NFD names.
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

- Every write goes through `ReplicatedDocumentStorage` / `ReplicatedCatalogRepository`, never to a single folder directly.
- Order: primary first, then each sync folder. If the primary fails, abort without touching the others. If a sync folder fails or is offline, record the operation in the **failed-operation journal** (local app data directory, never inside a library folder) and continue with the next folder.
- All catalog copies carry the same `CatalogRevision` after a successful write. On load, the higher revision wins; equal revisions with different content, and conflict copies, are merged by `CatalogMerger` per document id and reported.
- **Sync to all destinations**: copy what is missing or different in sync folders from the primary, apply catalog-known renames and moves. Files that exist only in a sync folder are reported, never removed.
- **Merge all folders**: scan every folder, copy each file missing anywhere into the folders that lack it. Same relative path with different content is a `ContentConflict` — never pick a winner automatically.
- `FolderComparison` compares by NFC relative path, then size, then hash (only when size and mtime do not settle it).

## Scanning

- Files in iCloud, Proton Drive and other sync clients may be **placeholders** that are not downloaded. Reading one triggers a download. Scans compare path, size and modification time first and hash only new or changed files.
  - macOS: iCloud may show evicted files as `.<name>.icloud`; map these back to their real name and treat them as "present, not downloaded". File Provider placeholders (Proton Drive) report their real size without local data.
  - Windows: cloud placeholders carry the `RECALL_ON_DATA_ACCESS` / `OFFLINE` attributes; do not read them during a scan.
- Sync clients create **conflict copies** (`dms-catalog 2.yaml`, `dms-catalog (conflicted copy).yaml`, …). Detect conflict copies of the catalog, merge them, and report them — never ignore them.
- Ignore: `.DS_Store`, `._*`, `Icon\r`, `Thumbs.db`, `desktop.ini`, `.dms-tmp-*`, `dms-catalog*.yaml`, and `.dms-text/` (handled separately).

## The `.dms-text/` store

- A hidden folder in every library folder: `.dms-text/<documentId>.txt`, UTF-8. Named by document id, so renames and moves do not touch it.
- It is derived data: written when a document is read for classification, replicated like any other write, and safe to rebuild.
- A missing or stale text file is never an error; search just finds less.
