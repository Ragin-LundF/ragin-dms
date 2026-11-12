# Changelog

All notable changes to ragin-dms are listed here.

## 2.1.0

A new "Organize" group in the navigation: To pay, Contracts, Tax years, Cases and Emergency folder. Everything
there works without AI; with AI switched on, it only adds suggestions you review. Nothing changes in your library
until you click. Looking at a new tax year shows a preview; it is saved only when you start the checklist.

### Bills to pay

- New "To pay" screen: every unpaid bill by due date, with totals for open, overdue and paid this month. Mark
  bills as paid, undo, and settle older bills in one step.
- A payment QR code (GiroCode) for each bill, read by any banking app.
- E-invoices (ZUGFeRD, Factur-X, XRechnung) inside PDFs are read exactly, also without AI. Their values are
  marked "E-invoice" in the review.
- The AI also reads the IBAN and payment reference of a bill. An invalid IBAN is refused in the review.
- An overdue bill appears once on the dashboard, with "Mark as paid" and "QR code". A paid bill no longer
  reminds you of its due date.
- New library filter "Payment": open, overdue or paid bills.

### Expected documents

- The app learns when senders send regular documents and tells you when one is late, for example "Arrived
  every January since 2021". Answer with Requested (asks again in 14 days), Ignore once, Stop expecting, or
  Draft request.

### Contracts and subscriptions

- New "Contracts" screen: every contract and subscription with its monthly and yearly price, the total per
  month and year, and how much more you pay than a year ago.
- The app proposes contracts from your documents (a cancel-by date, insurance or contract categories, regular
  bills); create one in a click or dismiss the proposal for good.
- Enter the term, renewal and notice period, and the app works out the last day to cancel and reminds you
  30 days ahead on the dashboard.
- Draft a cancellation letter right from the contract.
- With AI switched on, the app reads prices and terms from price change letters; you accept each price on its
  own, and increases are marked in the price history.
- Quick look shows the contract a document belongs to, or adds it to one.

### Tax years

- A new screen lists what the tax return of a year needs, built from a standard list, last year's tax
  documents and senders that send every year. Each item shows whether it was found, is still expected or is
  missing. Mark an item as "I have it", request it with a letter, or mark it as not needed this year.
- Amounts are added up per tax area.
- "Send to tax adviser" exports the year as a ZIP grouped by tax area, with a page and a spreadsheet of what is
  still missing.
- Documents can be assigned to a tax year and area in the quick look and for many at once from the selection
  bar. The library has a new "Tax year / area" filter, and the ready-made "Tax" smart folder now uses the tax
  year (tagged documents still count).
- With AI on, suggestions for tax documents also propose the tax year, the area and the labour share of
  craftsmen's bills.

### Cases with follow-ups

- Keep the letters back and forth about one matter together in a case, with its status, the next step, notes
  and follow-ups. A follow-up shows on the dashboard on its day and can be marked as done or snoozed for
  7 days. A case that has waited three weeks without an answer appears on the dashboard too, with a button to
  draft a reminder.
- When a letter asks you to do something, the AI proposes it as a follow-up with the requested date, along
  with matching documents from your library. A new letter from someone you have an open case with can be added
  to that case right in the review.
- Any document can join a case from its quick look.

### Letters

- Write letters from your documents: a cancellation, objection, reminder, request or reply is drafted for you
  (by your AI when it is switched on, otherwise from a ready-made template). You edit it and save it as a PDF
  into the folder of your choice. The letter is filed with its subject, recipient and today's date, put into
  the category "Letter" if you have one, and linked to the documents it refers to. A letter can start a case
  or go into one, with a follow-up after a number of days.
- Your name and address for letters are set once in Settings ("Your details").
- Text PDFs, such as letters, keep umlauts and the euro sign and can be searched.

### Emergency folder

- Collect the documents, contracts, contacts and notes a partner or relative would need. Switch each one on
  or off, print an overview or save it into the library, and create an encrypted ZIP (AES-256) with a password
  for a trusted person. Open it with 7-Zip, Keka or The Unarchiver; the Finder's Archive Utility cannot open
  encrypted ZIPs of this kind.
- The app reminds you two weeks before the yearly refresh and marks documents that changed since then.

### Archive health

- Ragin-DMS reads every file in every library folder again and compares it with its stored checksum. It runs
  weekly by default (or monthly, or off) in the background when the library opens, and you can also start it
  with "Check now" or from the menu. A check that started on its own steps aside whenever you change something in
  the library and continues afterwards where it stopped.
- A damaged copy is repaired from an intact one in another folder; the damaged file goes to the trash. Files
  your cloud service keeps only online are skipped, never downloaded.
- The catalog is now kept as a dated snapshot after every change on this device (the last 20, one per day for
  a month, one per week for a year). You can compare a snapshot with today's catalog and restore it; your files
  stay untouched.

### Email import

- Import `.eml` files from the inbox ("Import email") or by dropping them onto the window. Each mail becomes a
  PDF with sender, date and text, and its attachments become documents of their own, linked to the mail.
- Optionally the inbox converts emails on its own, and the original email file can be kept.
- Mails dragged straight out of Apple Mail cannot be received; drag them to the Finder or the inbox folder
  first.

### AI

- Split scanned stacks: the AI proposes where each document in a scanned pile starts. Check the cuts on a page
  strip, click between two pages to add or remove one, then split. Every part waits for your review with the
  AI's title, category and sender as a suggestion. Available in quick look and the command palette, and in the
  inbox for scans with 3 or more pages once you switch that on in Settings › Inbox.
- Search in plain words: type a sentence such as "Stadtwerke bills 2025 over 50 euros" and choose "Turn into
  filters" (⌘/Ctrl+Enter). The AI turns it into ordinary filters, marked in violet, that you can see and
  remove. Names your library doesn't know are reported, never guessed.
- Both are hidden while "Use AI" is off.

### Catalog

- Contracts, cases, tax years, your details and the emergency folder are stored in the catalog file as new,
  optional entries. A catalog without them is written exactly as before, and version 2.0 and 1.3 keep the new
  entries when they save the catalog.
- When two copies of the catalog disagree about a contract, case, tax year, your details or the emergency
  folder, the newer one is kept and the difference is listed under "Review catalog merge", as for documents.

### Good to know

- The open tasks and settings of this device keep a few new entries (snoozed expected documents, the
  e-invoice, tax and follow-up parts of a suggestion, the archive check schedule). Version 2.0 can still read
  these files, but if 2.0 saves them on the same device, the new entries are lost: snoozes come back and those
  suggestion parts are gone. Your library and its catalog are not affected.
- When the AI drafts a letter, it gets the documents' text but not your name or address; the app adds those to
  the letter itself.
- The encrypted emergency ZIP hides the content of the files, not their names: whoever has the ZIP can see the
  file names without the password.

## 2.0.0

### Rewritten in Rust and Tauri

- The whole app is rewritten in Rust with Tauri. It looks and works as before.
- Much smaller download: the app is about 39 MB instead of about 150 MB.
- It needs less memory and starts faster.
- Same catalog and settings files, and version 1.3 can still open a library that 2.0 has used. Only the hidden
  search texts and the Ask index move into subfolders (see below), so 1.3 no longer finds them there: its text
  search finds less, and it builds its Ask index again.
- A new Edit menu with undo, cut, copy, paste and select all.

### Hidden files in subfolders

- The search texts in `.dms-text/` and the Ask index in `.dms-index/` are now kept in small subfolders, named
  after the first two characters of each document's id, so no folder holds thousands of files.
- When a library is opened, its existing files are moved there once, before the scan ("Moving search texts into
  subfolders"). Nothing is overwritten or deleted: a file whose subfolder already holds one stays where it is and
  is listed in the log. A sync folder that is offline is moved when it is next reachable.
- Thumbnails are saved beside the search text of a document, so they no longer have to be drawn again after every
  start. A thumbnail is saved only while it is smaller than a tenth of its document; others are kept in memory
  only. Thumbnails are saved in every library folder that is reachable at that moment.
- A thumbnail now changes when its document changes. Before, it showed the old content until the app was restarted.

### PDF engine

- PDFs are now read, rendered and written with PDFium, the PDF engine of Chrome. It is faster and opens more
  damaged PDFs.
- Text layers from "Make searchable" now keep umlauts, the euro sign, Greek, Cyrillic and other characters
  searchable. For this, the PDF embeds the full font once, which makes it about 200 KB larger.
- "Convert to real PDF" files are about 200 KB larger per font style used, because the full font is embedded.
- Text read from PDFs can have slightly different spaces and line breaks, so the AI and search may see the
  text a little differently.
- Page pictures and thumbnails can look slightly different, and small thumbnails are sharper.

### Office files

- Word files are read without headers, footers and footnotes, and PowerPoint files without speaker notes.
- Numbers in Excel files are read as plain numbers, without their cell formatting.

## 1.3.0

### Turn scans into real PDFs

- New "Convert to real PDF" for scanned PDFs that have no text yet. You find it in a document's PDF
  tools in quick look and in the command palette.
- Your AI model reads every page. Each line it can read without doubt becomes real text that you can
  select, copy and search. Logos, stamps, signatures, handwriting, tables and every line the AI is not
  sure about stay exactly as they were scanned.
- Crooked scans are straightened first.
- Compare the original and the new PDF side by side, page by page. Flip between them, or mark what
  became text and what stayed an image. You also see both file sizes and why some lines stayed images.
- Then choose: keep the original, use the new PDF (the scan goes to your system's trash), or keep both.
  The new PDF is then added next to the original, with the same category, tags and fields.
- Your library only changes when you choose. If the PDF changes while it is being converted, or
  before you choose, the original is not replaced.
- This needs an AI model that can read images. A local model takes a few minutes per page. You can
  cancel at any time, and a page the AI cannot read stays as it was.

### AI

- Use Anthropic's Claude as an alternative to your local AI server. The API key is read from the
  ANTHROPIC_API_KEY environment variable and is never saved. Ask still builds its index with your local
  server.
- A new "Use AI" switch in Settings › AI. While it is off, the app shows no AI features at all. It is
  off on a new computer and stays on if you already use AI.

### Settings

- Settings show one section at a time, chosen from a list: Library folders, Scanning, Inbox, Custom
  fields, Matching rules and AI. Changes you have not saved yet are kept when you switch sections.

## 1.2.0

### A new look for the library

- Optimized the whole app for a calmer, clearer look, with a new font and matching light and dark
  themes.
- The library has a new sidebar with All documents, Inbox, Needs review, Added this month, Folders and
  your saved views. You can resize it or fold it into a narrow bar, and the app remembers your choice.
- Search and filter in one place: type to search, and narrow the results by category, tag, year,
  sender, type or folder. Save any combination as a view.
- See your documents as page previews or as a list, grouped by month, category or sender.
- Select several documents at once to give them a category or tag, move them or put them in the trash.
- Folders work like a file manager: a folder tree, new folders named in place, and folders you can
  rename, move by dragging, search in and delete. Before deleting, the app tells you exactly what goes
  to the trash.
- Quick look shows a document's preview and key facts beside the list. You can edit them right there
  and accept the AI's suggestions in one click.
- Work faster with the keyboard: "/" to search, Space for quick look, the arrow keys to go to the next
  or previous document, and more. Every shortcut is also listed in the new "Documents" menu.
- The Trash entry is gone from the sidebar. Deleted documents are in your system's trash.

### Safer with several computers

- Saving the catalog no longer overwrites changes another computer saved in the meantime. The app
  combines both versions and then applies your change again.
- An additional folder that holds a newer catalog is no longer overwritten with an older one.
- A file that appears at the target while the app copies or saves another file there is never
  replaced. The copy stops and the existing file stays as it is.
- If the catalog cannot be saved during a restructure, the files that already moved are kept in the
  record. You are told to run a scan, and reversing the restructure also moves them back.
- Values written by a newer version of the app are kept in rules, smart folders, fields, links,
  retention and the library settings when an older version saves.

### AI

- Choose how much of a document's text is sent to the AI, from 1,000 to 200,000 characters, in the
  AI settings. Smaller local models need less, larger ones can take more. The default stays 12,000.
- A broken or unexpected answer from the AI server is reported as an error instead of stopping the
  action.
- A server that takes too long to build the index for Ask is reported as a timeout.
- The documentation now says which servers work: OpenAI-compatible servers that need no API key, such
  as LM Studio and Ollama.

### Rules

- A rule's target folder always wins over the folder the AI suggests and covers its subfolders. A
  document that is already in the folder or below it stays where it is.

### Fixes

- A document keeps the value of a field whose definition was removed and can still save its other
  fields.
- A question in Ask that is stopped no longer keeps searching forever.
- A refused export keeps its settings, so you can correct them instead of starting over.
- A path template only counts as saved once it really was saved.
- A blank category name is refused with a message. Renaming or removing a category or tag only
  changes the selection once the change is saved.
- The page editor closes with a message when no library is open.
- Dates are checked the same way everywhere, with one message.

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
