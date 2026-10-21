# Changelog

All notable changes to ragin-dms are listed here.

## 1.0.0 - 2026-10-21

The first release of ragin-dms: a desktop document management system that works directly on your
own folders, without Docker, a database or a server, and uses your own local AI.

### Your documents, your folders

- Works on plain directories. Your documents stay ordinary files in your own folder structure and
  remain fully usable without the app.
- Categories, tags, titles, document dates and correspondents are saved in a readable catalog file
  next to your documents.
- Nothing is overwritten silently and nothing is deleted for good. Files only go to the trash, and
  only after you confirm.
- Runs on macOS, Windows and Linux, with installers for each.

### Several copies of your library

- Keep the library in one main folder plus any number of additional folders, for example a local
  disk, an external drive, iCloud or Proton Drive.
- Every change you make in the app is applied to all folders. Changes for a folder that is offline
  are remembered and applied once it is back.
- "Sync to all" brings every folder up to date with the main folder, and "Merge" collects files that
  are missing anywhere. Neither ever deletes anything.
- Moved and renamed files are recognized as such instead of showing up as new.
- When a file has a different name in one of the folders, one click renames it to match.
- Conflicting copies of the catalog created by sync clients are merged automatically.

### Dashboard

- One place for everything that needs your decision: new files, AI suggestions to review, files
  changed, moved or missing outside the app, differing copies, duplicates and failed operations.
- Thumbnails of the documents in open tasks.
- Statistics about your library: number of documents, storage, how much is categorized, documents
  per year, categories, tags, correspondents and recently added documents.
- Click a year to see that year's documents in the library.
- Safe removal of duplicates: files only count as duplicates when they are truly identical. The extra
  copies go to the trash in every folder, and their categories and tags are kept on the remaining copy.

### Library

- Browse by folder, category or tag in collapsible trees, and filter by year.
- List and grid views with thumbnails of PDFs and images.
- Add files by dragging them into the app, and move or categorize documents by drag and drop.
- Reorganize your categories by dragging them onto each other.
- Edit title, date, correspondent, category and tags of any document, with tag suggestions as you type.
- Open a document's location in Finder or Explorer.
- Search the contents of your documents, including scanned documents the AI has read.

### Local AI

- Works with your own local AI server, such as LM Studio or Ollama. Your documents never leave your
  computer.
- Suggests title, category, tags, document date, correspondent and file name. File name suggestions
  follow the naming style already used in your library.
- Reads PDFs, scans and photos, and Word, PowerPoint and Excel files.
- Every suggestion is reviewed before anything changes. Edit any value, pick a category or create a
  new one right in the review.
- Renaming files to the suggested name is optional and can be switched on in the settings.
- Categorize or tag all new documents in one go, in the background.
- Ask questions about a document or get a short summary of it.

### Settings

- Choose your library folders per computer.
- Optionally fill in missing document dates from the file name.
- Choose whether new files are added while scanning or first listed on the dashboard.
- Configure the AI server, test the connection and pick one of its models.
