# ragin-dms

**A document management system for people who already have their folders in order.**

Most document management systems want you to set up Docker, run a database, and then import your
documents into their own storage, after which your files are only really usable through the DMS.
ragin-dms takes the opposite approach: it is a plain desktop app that works directly on the folders
you already have. There is no server, no database and no import. Your documents stay ordinary files
in your own directory structure, and you can keep using Finder, Explorer, your backup tool or your
cloud sync client exactly as before.

On top of those folders, ragin-dms lets you tag and categorize documents, search inside them, and keep
several copies of the library (for example a local folder plus iCloud or Proton Drive) in sync.
Categories and tags are stored in a small, readable file (`dms-catalog.yaml`) right next to your
documents, so nothing is hidden away and nothing is lost if you stop using the app.

To save you the typing, ragin-dms can suggest titles, categories, tags, dates and file names with
**your own local AI**, such as LM Studio or Ollama. Your documents are read on your machine and are
never sent to a cloud service. Every suggestion waits for your approval before anything changes.

Runs on macOS, Windows and Linux. Built with Kotlin and Compose Multiplatform.

![The dashboard of a demo library](docs/screenshot.png)

<sub>The screenshot shows a demo library with invented documents, categories and tags.</sub>

## Why ragin-dms

* **No setup.** Install the app, choose your folder, done. No Docker, no database, no web server.
* **Your structure stays.** Files are never moved into a hidden store; the library is your directory tree.
* **Private by design.** AI runs on your own machine through any OpenAI-compatible local server.
* **Safe.** Nothing is overwritten silently, and nothing is deleted for good: files only go to the
  system trash, and only after you confirm.

## Features

### Your files stay yours

* The library is a plain directory tree. Every file manager, backup tool and sync client keeps
  working, and the library stays usable without the app.
* Categories, tags, titles, document dates and correspondents are stored in `dms-catalog.yaml` in the
  library root: human-readable, diffable, and copied into every library folder.
* Nothing is overwritten silently and nothing is deleted for good. Files only ever go to the system
  trash, and only after confirmation.

### Several library folders

* One primary folder plus any number of sync folders, each just a path (local disk, external disk,
  network share, or a folder that iCloud, Proton Drive or another client backs up). The app has no
  vendor-specific logic.
* Every change made in the app (add, rename, move, create folder, trash) is applied to all reachable
  folders. Operations for an offline folder are journaled and retried when it is back.
* **Sync to all** makes every sync folder match the primary; **Merge** copies every file that is missing
  anywhere into the folders that lack it. Neither deletes anything.
* Moved and renamed files are recognized by their SHA-256 hash. Paths are compared NFC-normalized, so
  macOS, Windows and Linux folders match.
* **Different names**: when a file has another name in one of the folders, one click renames it to match.
* Conflict copies of the catalog written by sync clients (`dms-catalog 2.yaml` and similar) are merged
  per document.

### Dashboard

* Everything that needs a decision is collected as an open task: new files, suggestions to review,
  files changed, moved or missing outside the app, different content in two folders, duplicates,
  catalog conflicts and failed operations. Open tasks show inline thumbnails.
* Statistics: documents, storage, how much is categorized, documents per year (click a year to open the
  library filtered to it), categories, tags sized by use, correspondents, and recently added documents.
* **Safe duplicate removal**: copies are only treated as duplicates when size, SHA-256 and a
  byte-by-byte comparison agree. The extra copies go to the trash in every folder, and their metadata
  is merged into the copy that is kept.

### Library

* Sidebar with collapsible trees for folders, categories and tags, and a year filter.
* List and grid layouts with thumbnails of PDFs and images.
* Drag and drop files from the file manager to add them, documents onto folders or categories to move
  or categorize them, and categories onto each other to restructure the category tree.
* Inspector with preview, metadata editor, tag editor with autocomplete, and "show in Finder/Explorer".
* Full-text search over the text of every document, including scans the AI has transcribed. The text
  is kept in the hidden `.dms-text/` folder of the library.

### Local AI

* Works with any OpenAI-compatible server that needs no API key, such as LM Studio (the default, at `http://localhost:1234/v1`) or Ollama. Providers that require an API key are not supported.
  The settings list the loaded models and have a connection test.
* PDFs are read from their text layer, or rendered and sent to a vision model when there is none (or
  always, with **Read PDFs as images**). Images go to the vision model, and Word, PowerPoint and Excel
  files are read with Apache POI.
* Suggestions for title, category, tags, document date, correspondent and file name. The naming
  convention is learned from the existing file names of the library.
* Every suggestion is reviewed before anything is written: edit any field, pick an existing category
  or create a new one. Renaming to the suggested file name is opt-in.
* **Classify with AI** and **Tag with AI** process all new or untagged documents in the background.
* Ask questions about a document in a chat, or let the AI summarize it.

## Modules

| Module | Purpose |
|---|---|
| `dms-domain` | Entities and domain rules. No dependencies. |
| `dms-application` | Use cases and ports. |
| `dms-storage` | File system, catalog file, synchronization of the library folders. |
| `dms-ai` | OpenAI-compatible LLM client, embeddings, reading PDFs, images and Office files. |
| `dms-pdf` | Writing PDF files: page tools, text layers, embedded metadata, installed OCR. |
| `dms-ui` | Compose Multiplatform desktop UI and composition root. |

Dependencies point inwards only: `dms-ui` → `dms-application` → `dms-domain`; the adapters
`dms-storage`, `dms-ai` and `dms-pdf` implement ports of `dms-application`.

The app keeps the state of each device (library folder paths, AI settings, open tasks, activity log,
scan caches) outside the library, in `~/Library/Application Support/ragin-dms` on macOS, `%APPDATA%\ragin-dms`
on Windows and `$XDG_DATA_HOME/ragin-dms` (default `~/.local/share/ragin-dms`) on Linux.

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
