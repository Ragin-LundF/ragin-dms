# PDF Engine and Document Reading Instructions

Use this file when touching `crates/infra-pdf` (`dms_infra_pdf`): reading documents for the AI and search, thumbnails and page pictures, page tools, text layers, embedded metadata, the rebuild of scans, or the installed OCR program.

The engine is PDFium through `pdfium-render` (rendering, text, writing pages) plus `lopdf` (metadata). Office files are read with `zip` + `quick-xml` (DOCX, PPTX) and `calamine` (XLSX).

## Module layout

| Module | Purpose |
|---|---|
| `engine` | `PdfEngine`: binds the PDFium library once per process (`bind`, from the app's resource folder; `dev_library()` for tests and `cargo run`). Every operation is synchronous; callers run it on a blocking thread. |
| `document` | Reading (`read_content`, `page_texts`, `has_text_layer`, `render_page`, `thumbnail`), page tools (`rewrite`, `merge`), the invisible text layer (`add_text_layer`) and metadata (`read_metadata` via PDFium, `write_metadata` via `lopdf`, document info and XMP). |
| `rebuild` | `Rebuilder`: the "Convert to real PDF" engine (deskew, page preparation, crops, composing text and kept image regions). |
| `images`, `fonts`, `files`, `ocr` | Vision JPEG and layout PNG scaling; the bundled Liberation fonts (`crates/infra-pdf/fonts/`); temporary files and placing results; `OcrMyPdf` for the installed `ocrmypdf`. |
| `adapter` | The port implementations, which only map types: `FileContentReader` (`DocumentContentReader`), `FilePreviewer` (`DocumentPreviewer`), `PdfiumEditor` (`PdfEditor`), `PdfiumRebuilder` (`PdfRebuilder`), `OcrMyPdfTool` (`OcrTool`), bundled in `PdfAdapters`; `LibraryLocations` resolves library paths. Office reading in `adapter::content`. |

## File safety rules

- Sources are only read. Every result is a new file, written to a temporary file next to its target and moved into place, **never over an existing file**. Replacing the user's PDF is a use case decision (the old file goes to the trash).
- Before a metadata write is placed, PDFium opens the result and must find the same page count; otherwise nothing is written.
- Rendered pictures and rebuild work files stay in memory or in the app data directory, never in a library folder.
- A failed operation leaves the source byte-for-byte unchanged. Expected failures are `PdfError` values mapped to the port outcomes; never panic on a damaged or encrypted PDF.

## Document reading rules

`DocumentContentReader::read` returns `DocumentContent`: `Text`, `PageImages`, or `NotReadable`. `DocumentContentReader::text` returns only text and never renders, so search needs no AI server.

| Input | Reader | Result |
|---|---|---|
| PDF with a usable text layer | PDFium text | `Text` (unless "prefer vision for PDFs" is on) |
| PDF without a text layer (scan) | PDFium rendering | `PageImages` |
| JPG, PNG (recognized by content, not extension) | downscale | `PageImages` |
| DOCX, PPTX, XLSX | `quick-xml` on `word/document.xml` and the slides in order; `calamine` for sheets | `Text` |
| anything else | — | `NotReadable`; never sent to the AI |

- "Has a usable text layer" is decided in one place (`document::text_layer`): at least 20 non-whitespace characters per page on average; the AI reader needs 80 characters over the first pages. Scanner OCR layers count as text layers. Tested with a synthetic text PDF and a synthetic image-only PDF.
- Render only the first pages (2) at 150 dpi, downscaled to at most 1600 px on the long side (`images::vision_jpeg`). Image size is what makes a local vision model slow; never send full-resolution pages.
- Text is cut to the configured input budget (`inputCharacters`) before it leaves the adapter.
- Reading never modifies the source file. Reading a cloud placeholder downloads it, so reading runs only for documents the user asked to classify or search.
- DOCX headers, footers and footnotes and PPTX notes are not read (a documented change from 1.x).

## PDFium rules and pitfalls

- PDFium binds once per process; keep the `Pdfium` in the engine's `OnceLock`. `scripts/fetch-pdfium.sh` pins the release (`chromium/8125`), which must match the `pdfium-render` feature `pdfium_8125`; change both together. The Tauri bundle ships the library from `vendor/pdfium/<platform>/`.
- `PdfPageText::all()` drops text on pages with `/Rotate`; read text character by character instead.
- Objects added through `page.objects_mut()` are lost with the automatic regeneration strategy; use `Manual` and call `page.regenerate_content()`.
- Text layers and rebuilt pages embed a bundled Liberation font as a CID font, so every character stays searchable. Do not switch back to a non-embedded standard font: it drops everything outside Latin-1.
- Rebuilt pages are produced by the same algorithm as 1.x (positions, sizes, colours, crops, deskew); keep it that way unless the task asks for a change.

## OCR

- "Precise OCR" uses `ocrmypdf` if it is installed and on the `PATH`; nothing is bundled. Its absence is a status the UI shows, not an error.
- The program runs without a shell, with arguments as a list, and writes a new target file that must not exist yet; the source is never changed in place. Process runs go through `CommandRunner`, so tests use a scripted runner.
