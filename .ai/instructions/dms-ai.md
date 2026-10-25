# AI Classification and Document Reading Instructions

Use this file when touching `dms-ai`: LLM providers, prompts, structured output, or reading documents (PDF text layer, rendered pages, images, Office files).

## Package layout

| Package | Purpose |
|---|---|
| `dms.ai` | `AiAdapters`: builds the HTTP client and hands out the client and the reader. Tests pass a Ktor `MockEngine`. |
| `dms.ai.client` | `ChatTransport`: the HTTP side of OpenAI-compatible APIs (LM Studio, Ollama's `/v1`, any key-less server): `/models`, `/chat/completions`, the model choice (`ModelChoice` skips embedding models when no model is set), error mapping, and `AnswerText`, which drops `<think>` blocks. `ChatRequestBuilder` builds the bodies with text and image parts. `OpenAiCompatibleClient` implements `DocumentClassifier` and `AiServer`. `OpenAiEmbeddings` implements `EmbeddingClient` (`/embeddings`; one vector per text, by index, all of one size, or a typed failure). Ktor wraps deserialization errors in `ContentConvertException`: catch it, so a malformed answer becomes a typed failure. DTOs in `dms.ai.client.dto`. |
| `dms.ai.assistant` | `OpenAiAssistant` (implements `DocumentAssistant`): summary, tags and questions about one document, free-text answers. `OpenAiLibraryAssistant` (implements `LibraryAssistant`) answers a question about the library from numbered passages with citations, using the `ask-library` prompt. `TagAnswerParser` turns a free-text tag answer into `TagNameModel`s. |
| `dms.ai.prompt` | The answer schema, the prompt texts, the naming convention detected from file names, the tax tag (`TaxTag`), and the mapping of the answer to a `ClassificationSuggestionModel`. `PromptTemplates` loads and fills the templates in `src/main/resources/prompts/` (`classification`, `summary`, `tags`, `chat`, `ask-library`, each with a `-request` user part where needed, plus the fragments `document-text`, `document-images` and `classification-outline*`/`classification-rules`). |
| `dms.ai.read` | `FileContentReader` (implements `DocumentContentReader`): PDF (PDFBox: text layer or rendered pages), images, Office (Apache POI). |

The provider settings (`AiSettingsModel`: base URL, model, timeout, "prefer vision for PDFs") are a port model in `dms-application` and are stored per device by `dms-storage`. The client reads them for every request, so a change in Settings applies at once.

## Provider rules

- Talk to providers through the OpenAI-compatible chat completions API with a configurable base URL. That one client covers LM Studio (`http://localhost:1234/v1`), Ollama (`http://localhost:11434/v1`) and any other OpenAI-compatible server that needs no API key, local or on the network.
- Local first: the default configuration points at LM Studio on `localhost`. Sending document content to a remote API requires the user to configure it explicitly; never fall back to a remote provider automatically.
- API keys are not supported: no request carries an `Authorization` header, so providers that need a key (such as OpenAI's own API) do not work. If keys are added later, store them in the OS credential store (macOS Keychain, Windows Credential Manager, Linux Secret Service), never in the catalog, in files inside a library folder, or in logs.
- Every request has a timeout and can be cancelled from the UI.
- Do not log document text, images or prompts at `INFO` or above.

## Classification rules

- Request **structured output** (JSON schema via `response_format`) and validate it against the schema before mapping. Invalid output becomes a typed failure, never a partly filled suggestion.
- Give the model the **existing taxonomy** (categories and tags from the catalog), the folder tree, and the library's **naming convention**, detected at runtime from the existing file names (never hard-coded). Instruct it to prefer existing values. New categories or tags are allowed but marked as new so the UI can highlight them.
- The classifier returns a `ClassificationSuggestionModel`. It never writes to the catalog or renames files — the user approves on the dashboard, then a use case applies it.
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
