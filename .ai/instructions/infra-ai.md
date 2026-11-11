# AI Provider and Prompt Instructions

Use this file when touching `crates/infra-ai` (`dms_infra_ai`): LLM providers, prompts, structured output, embeddings, or the page layout reader of "Convert to real PDF". Reading documents (PDF text, page pictures, images, Office files) lives in `crates/infra-pdf`; see `infra-pdf.md`.

## Module layout

| Module | Purpose |
|---|---|
| `ai_adapters` | `AiAdapters`: hands out the client, the assistants, the embeddings and the layout reader. `new(settings)` reads the Anthropic key from the environment; `with_anthropic_key` injects a key source (tests). |
| `client` | `ChatTransport`: the HTTP side (`ureq`) of OpenAI-compatible APIs (LM Studio, Ollama's `/v1`, any key-less server) and Anthropic: `/models`, `/chat/completions`, `/messages`, the model choice (embedding models are skipped when no model is set), error mapping, and `AnswerText`, which drops `<think>` blocks. `chat_request` builds the bodies with text and image parts, `anthropic_request` maps them to the Messages API. DTOs in `client::dto`. |
| `classify` | `OpenAiCompatibleClient` (implements `DocumentClassifier` and `AiServer`): the answer schema (`suggestion_schema`), the prompt texts, the naming convention detected from file names, and the mapping of the answer to a `ClassificationSuggestion` (`suggestion_mapper`, `SuggestionDto`). |
| `assistant` | `OpenAiAssistant` (implements `DocumentAssistant`): summary, tags and questions about one document, free-text answers. `OpenAiLibraryAssistant` (implements `LibraryAssistant`) answers a question about the library from numbered passages with citations, using the `ask-library` prompt. `tag_answer_parser` turns a free-text tag answer into `TagName`s. |
| `embedding` | `OpenAiEmbeddings` (implements `EmbeddingClient`): `/embeddings`; one vector per text, by index, all of one size, or a typed failure. |
| `prompt` | The prompt templates, compiled in with `include_str!` from `crates/infra-ai/prompts/` (`classification`, `summary`, `tags`, `chat`, `ask-library`, each with a `-request` user part where needed, the fragments `document-text`, `document-images`, `classification-outline*`/`classification-rules`, and `rebuild-layout*`/`rebuild-line`), and the tax tag (`tax_tag`). |
| `rebuild` | `AiPageLayoutReader` (implements `PageLayoutReader`): the layout image, schema, DTOs and mapper for "Convert to real PDF". |

The provider settings (`AiSettings`: provider, base URL, model, timeout, "prefer vision for PDFs", enabled) are a port model in `dms_application::port::state` and are stored per device by `crates/infra-storage`. The adapters get a settings source (`Arc<dyn Fn() -> AiSettings>`) and read it for every request, so a change in Settings applies at once.

## Provider rules

- Two providers, chosen by `AiSettings::provider`. Local: the OpenAI-compatible chat completions API with a configurable base URL, covering LM Studio (`http://localhost:1234/v1`), Ollama (`http://localhost:11434/v1`) and any other OpenAI-compatible server that needs no API key. Anthropic: the native Messages API at the fixed `https://api.anthropic.com/v1`; `ChatTransport` sends `/messages` and maps the chat completions body to it. Embeddings always use the local server, because Anthropic has none.
- Local first: the default configuration points at LM Studio on `localhost`. Sending document content to a remote API requires the user to configure it explicitly; never fall back to a remote provider automatically.
- The only API key is Anthropic's. By the user's decision it is read from the `ANTHROPIC_API_KEY` environment variable only: never stored (not in settings, the catalog, a library folder or the OS credential store), never logged, and sent only to `api.anthropic.com`, never to the local server. Other providers that need a key (such as OpenAI's own API) are not supported.
- Every request has a timeout and can be cancelled from the UI.
- Never log or print document text, images, prompts or answers. Error details shown to the user are shortened and never contain document content.
- HTTP and JSON errors (unreachable server, non-JSON body, wrong shape, empty body) become typed failures, never panics.

## Classification rules

- Request **structured output** (JSON schema via `response_format`) and validate it against the schema before mapping. Invalid output becomes a typed failure, never a partly filled suggestion.
- Give the model the **existing taxonomy** (categories and tags from the catalog), the folder tree, and the library's **naming convention**, detected at runtime from the existing file names (never hard-coded). Instruct it to prefer existing values. New categories or tags are allowed but marked as new so the UI can highlight them.
- The classifier returns a `ClassificationSuggestion`. It never writes to the catalog or renames files — the user approves on the dashboard, then a use case applies it.
- Suggested file names go through the domain's `FileNamePolicy` before they are shown.
- When images were sent, the schema also asks for a plain-text transcription; it is stored in `.dms-text/` for search.
- Keep prompts in `crates/infra-ai/prompts/*.md`, not in string literals, so they can be read and tuned without touching code.
- Truncate document text to the configured input budget before sending; say in the prompt that the text may be truncated.
- `PageImages` require a vision-capable model. If the configured model cannot take images, return a typed failure that says so — do not silently send nothing.
- Images are sent as base64 `image_url` content parts (Anthropic: image blocks). Do not depend on a provider accepting PDF or Office files directly; LM Studio does not.
- Batch classification (hundreds of documents) runs in the background, reports progress, and can be cancelled.

## Rebuilding scans ("Convert to real PDF")

`rebuild::AiPageLayoutReader` implements `PageLayoutReader` for the rebuild use case (`dms_application::pdf::rebuild`) and `crates/infra-pdf`.

- The page goes out as lossless PNG, at most 1568 px on the long side and both sides a multiple of 28 (the patch size of Qwen-VL models), from `rebuild::layout_image`. Classification keeps its JPEG at 1600 px.
- The prompts are `rebuild-layout.md` (system), `rebuild-layout-request.md` (the sent size) and `rebuild-line.md`. The answer is constrained by the strict layout schema: `lines[]` with `bbox`, `text`, `bold`, `font` (sans/serif/mono) and `legible`, and `graphics[]` with `kind` and `bbox`.
- Boxes are asked for in 0..1000. If any value of an answer is above 1000, the whole answer is in pixels of the sent image (`layout_mapper`). Boxes leave the adapter in 0..1000 of the page.
- Layout requests wait at least 600 s (`LAYOUT_TIMEOUT_SECONDS`), whatever the configured timeout: a 27B model needs minutes per page. A timeout fails only that page, which then stays as it was.
- NUL repair: under a JSON grammar some models write `\u0000` for every non-ASCII character. `RebuildPdf` reads such a line again with `read_line`, as plain text without `response_format`, from a crop of the page, and takes the new reading only if `line_repair` finds every known character in place (each run of n lost characters may become 1..n characters). Otherwise the line stays an image.
- Never log the layout or line texts.
- `tests/lm_studio.rs` sends a synthetic invoice to the running model; it runs only with `DMS_AI_INTEGRATION=true` (optionally `DMS_AI_BASE_URL`, `DMS_AI_MODEL`).
