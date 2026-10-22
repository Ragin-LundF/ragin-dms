# Changelog

All notable changes to ragin-dms are listed here.

## 1.1.0 - 2026-10-22

### Inbox

- An optional inbox folder inside your library, switched on in the settings. Drop new documents into
  it on any computer and they show up in the app, even when they arrive in one of the additional
  folders.
- For each document choose: let the AI analyze it, or move it to a folder yourself, without any AI.
- The AI can also suggest the folder a document belongs in, with a short reason, including a new
  subfolder. You approve the folder together with the other values.
- Optionally analyze new inbox documents automatically.
- An "Inbox" page and an inbox card on the dashboard with "Analyze all".

### Restructure your folders

- Sort documents into folders by a pattern, such as category and year, for the whole library, one
  folder or a smart folder.
- See every planned move before anything happens, leave out folders you want to keep, and apply the
  rest in one go, with progress.
- Every restructure can be reversed later. Folders left empty can be moved to the trash.
- The dashboard points out folders with many loose files.

### Custom fields and deadlines

- Add your own fields to documents, such as amount, due date, cancellation date, contract end or
  warranty, with ready-made standard fields to start with.
- The AI fills in the fields it finds in a document, for you to approve.
- Upcoming deadlines appear on the dashboard in good time, and an "Upcoming" card lists the next
  dates. Export them as a calendar file.
- Retention periods are entirely up to you: set a keep-until date on a document or a period on a
  category. When it ends, the dashboard asks whether to move the document to the trash or to keep it.

### Notes and linked documents

- Write a note on any document. Notes are found by the search.
- Link documents that belong together, such as an invoice and its reminder.

### Smart folders

- Save any view of the library as a smart folder, with the number of documents it holds.
- Ready-made smart folders for untagged and uncategorized documents, this year's tax documents and
  what is due in the next 30 days.

### Rules

- The app learns from the documents you approved and proposes rules, such as "documents from this
  sender are invoices". Accept or reject each proposal.
- Rules fill in category, tags and other values without the AI, and the review shows which rule a
  value came from.
- Create a rule from a document with one click, and switch rules on or off in the settings.

### Export

- Export documents, for example for your tax adviser, into a folder or a ZIP file, with an overview
  as a spreadsheet and a web page.
- Export the current view, a smart folder, a tax year or everything from one sender. Your library
  stays untouched.

### PDF tools

- Rotate, remove and reorder the pages of a PDF, and split it into several documents.
- Merge several PDFs into one.
- Make scanned PDFs searchable, using the text the AI read or an installed OCR program.
- Optionally write title, sender, category, tags and date into the PDF files, so other applications
  see them too.
- Replaced versions of a file always go to the trash first.

### Ask your library

- Ask questions about all your documents and get answers with the documents they come from.
- Ask within the whole library or one smart folder.
- See related documents for every document.
- Everything runs on your own local AI. The search index is kept with your library and synced to all
  its folders.

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
