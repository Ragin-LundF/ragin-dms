# Testing Instructions

Use this file when adding, modifying, or reviewing tests.

## General test rules

- Rust: plain `#[test]` functions with `assert_eq!`, `assert!` and `matches!`. TypeScript: Vitest.
- Cover positive paths, negative paths, edge cases, and error paths.
- Add or update tests for behavior changes. Never weaken, skip, or delete a test to make a change pass.
- Keep test names descriptive and behavior-focused (`a_save_over_a_newer_revision_is_refused`).
- No `unwrap()` in tests either; use `expect("what was expected")`.
- Never `std::thread::sleep` to wait for a result. Make the code under test observable (a channel, a recording fake) and wait on that.
- Commands: `cargo test --workspace` (all Rust tests), `cargo test -p <crate> <filter>` (a subset), `npm --prefix ui test` (Vitest).

## Domain and use case tests

- `dms-domain` tests are plain unit tests without I/O, in one test binary `crates/domain/tests/spec/` that mirrors the domain modules.
- `dms-application` tests live in one binary `crates/application/tests/spec/`, one folder per feature. They use hand-written **fakes** of the ports in `tests/spec/support/fake/` (in-memory `DocumentStorage`, `CatalogRepository`, a scripted `DocumentClassifier`, …), so each port has one fake. `TestLibrary` (`support/test_library.rs`) holds the fakes; each feature wires its use cases in `support/wiring/<feature>.rs`.
- Prefer fakes over mocking libraries. A fake documents the port's contract; a mock only repeats the test's expectations.

## Storage tests

- Every file system test works in a fresh temporary directory (`tempfile`), deleted afterwards. Never touch the user's home folder, the real app data directory, or any real library folder.
- Synchronization tests use **at least three** temporary folders (primary + two sync folders) and assert the state of every folder after each operation.
- Test failure paths explicitly: a sync folder not writable or missing (offline), file already exists, same path with different content, catalog file corrupt, catalog conflict copies present, file missing in one folder.
- Test names with umlauts in both NFC and NFD form, and names that differ only by case. (APFS resolves composed and decomposed names itself, so some NFC fallbacks are only reachable on other file systems.)
- Tests use a recording `SystemTrash`, never the system trash.
- Fixtures are **synthetic**. Never copy real documents or real file names of the user into the repository.
- Catalog format tests load real fixture files from `crates/infra-storage/tests/fixtures/catalog/`. When the format version changes, keep the old fixture and add a migration test that reads it.
- **Golden files**: `crates/infra-storage/tests/fixtures/jackson/` holds files written by the released 1.3.0 version (catalogs, settings, tasks, activity, journal, scan cache, plans, vector index, export index). Tests compare the Rust output with them byte for byte. Never "update" a golden file to match new output; a difference is a compatibility bug unless it is a deliberate, documented format change. How they were generated is in that folder's `README.md`.
- Round-trip every catalog change: write, read back, compare with `assert_eq!`.

## AI tests

- Unit tests never call a real LLM. Test the adapter against recorded or hand-written responses served by a local stub server (a `TcpListener` in `dms_infra_ai`'s test support).
- Test that malformed, partial, or off-schema model output becomes a typed failure, not a panic and not a half-filled suggestion.
- Integration tests against a running LM Studio are opt-in only: `DMS_AI_INTEGRATION=true cargo test -p dms-infra-ai --test lm_studio` (optionally `DMS_AI_BASE_URL`, `DMS_AI_MODEL`). Without the variable they do nothing.
- Tests never read `ANTHROPIC_API_KEY` from the environment; they inject a key source.

## PDF tests

- PDF tests need the PDFium library from `scripts/fetch-pdfium.sh`; they load it with `PdfEngine::dev_library()`.
- Build input PDFs and images in the test (synthetic pages, generated text), never from real documents.
- Check what a user would notice: text read back, page count, page order, the text layer, metadata, and that the source file is unchanged after a failed operation.

## Desktop app tests

- Presenters in `src-tauri/src/features/<area>/` are pure functions; test them with a synthetic catalog in a `#[cfg(test)] mod tests` beside them.
- `src-tauri/src/e2e_tests.rs` runs the main flows with the real storage and PDF adapters, an isolated app data directory, a recording trash and a stub AI. `prepare_synthetic_library` is `#[ignore]`d and only builds a scratch library for manual runs (`DMS_SYNTHETIC_ROOT=<scratch dir>`).
- UI logic in TypeScript gets Vitest tests (`*.test.ts` beside the file) only where it is non-trivial (state machines, formatting, selection). Do not test what Rust presenters already compute.
- `npm --prefix ui run dev:mock` shows the UI with synthetic mock data (`ui/src/mocks`) for visual checks; it never talks to a real library.
