# AI Classification and Document Reading Instructions

Use this file when touching `dms-ai`: LLM providers, prompts, structured output, or reading documents (PDF text layer, rendered pages, images, Office files).

## Package layout

| Package | Purpose |
|---|---|
| `dms.ai.openai` | Client for OpenAI-compatible chat completion APIs (LM Studio, Ollama's `/v1`, OpenAI), text and image content parts, `/v1/models`. DTOs in `dms.ai.openai.dto`. |
| `dms.ai.classification` | `LlmDocumentClassifier` (implements `DocumentClassifier`), prompt building, response parsing and validation. |
| `dms.ai.content` | `DocumentContentReader` implementations: PDF (PDFBox: text layer or rendered pages), images, Office (Apache POI), plain text. |
| `dms.ai.config` | Provider settings: base URL, model, timeouts, input budget, "prefer vision for PDFs". |

## Provider rules

- Talk to providers through the OpenAI-compatible chat completions API with a configurable base URL. That one client covers LM Studio (`http://localhost:1234/v1`), Ollama (`http://localhost:11434/v1`) and remote APIs.
- Local first: the default configuration points at LM Studio on `localhost`. Sending document content to a remote API requires the user to configure it explicitly; never fall back to a remote provider automatically.
- API keys are stored in the OS credential store (macOS Keychain, Windows Credential Manager, Linux Secret Service), never in the catalog, in files inside a library folder, or in logs.
- Every request has a timeout and can be cancelled from the UI.
- Do not log document text, images or prompts at `INFO` or above.

## Classification rules

- Request **structured output** (JSON schema via `response_format`) and validate it against the schema before mapping. Invalid output becomes a typed failure, never a partly filled suggestion.
- Give the model the **existing taxonomy** (categories and tags from the catalog), the folder tree, and the library's **naming convention** (`YYYY-MM-DD <Title>`, detected from existing file names). Instruct it to prefer existing values. New categories or tags are allowed but marked as new so the UI can highlight them.
- The classifier returns a `ClassificationSuggestion`. It never writes to the catalog or renames files — the user approves on the dashboard, then a use case applies it.
- Suggested file names go through the domain's `FileNamePolicy` before they are shown.
- When images were sent, the schema also asks for a plain-text transcription; it is stored in `.dms-text/` for search.
- Keep prompts in resource files (`src/main/resources/prompts/*.md`), not in string literals, so they can be read and tuned without touching code.
- Truncate document text to the configured input budget before sending; say in the prompt that the text may be truncated.

## Document reading rules

`DocumentContentReader` returns `DocumentContent`, a `sealed interface`: `Text` or `PageImages`. Unsupported kinds return a typed "not readable" result.

| Input | Reader | Result |
|---|---|---|
| PDF with a usable text layer | PDFBox `PDFTextStripper` | `Text` (unless "prefer vision for PDFs" is on) |
| PDF without a text layer (scan) | PDFBox `PDFRenderer` | `PageImages` |
| JPG, PNG | downscale | `PageImages` |
| DOCX, PPTX, XLSX | Apache POI (`XWPF`, `XSLF`, `XSSF` extractors) | `Text` |
| anything else | — | not readable; never sent to the AI |

- "Has a usable text layer" is decided in one function with a configurable minimum of characters per page, and tested with a synthetic text PDF and a synthetic image-only PDF. Scanner OCR layers count as text layers.
- Render only the first pages (default 2, configurable) at about 150 DPI, downscaled to a maximum long side (default 1600 px). Image size is what makes a local vision model slow; never send full-resolution pages.
- `PageImages` requires a vision-capable model. If the configured model cannot take images, return a typed failure that says so — do not silently send nothing.
- Images are sent as base64 `image_url` content parts. Do not depend on a provider accepting PDF or Office files directly; LM Studio does not.
- Reading never modifies the source file. Rendered images stay in memory or in the app's cache directory, never in a library folder.
- Reading a cloud placeholder downloads it; reading runs only for documents the user asked to classify.
- Batch classification (hundreds of documents) runs in the background, reports progress, and can be paused and cancelled.
