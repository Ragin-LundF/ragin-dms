# Testing Instructions

Use this file when adding, modifying, or reviewing tests.

## General test rules

- Use `kotlin.test` annotations and assertions when possible.
- Use named arguments for assertions.
- Cover positive paths, negative paths, edge cases, and error paths.
- Add or update tests for behavior changes.
- Keep test names descriptive and behavior-focused.
- Use `kotlinx-coroutines-test` (`runTest`) for `suspend` code. Never use `Thread.sleep` or `runBlocking` in tests.

## Domain and use case tests

- `dms-domain` tests are plain unit tests without I/O.
- `dms-application` tests use hand-written **fakes** of the ports (in-memory `DocumentStorage`, `CatalogRepository`, a scripted `DocumentClassifier`). Put fakes in a shared test fixtures package (`dms.application.fake`) so each port has one fake.
- Prefer fakes over mocking libraries. A fake documents the port's contract; a mock only repeats the test's expectations.

## Storage tests

- Every file system test works in a fresh temporary directory (`kotlin.io.path.createTempDirectory`) and deletes it afterwards. Never touch the user's home folder, iCloud, or Proton Drive paths.
- Synchronization tests use **at least three** temporary folders (primary + two sync folders) and assert the state of every folder after each operation.
- Test failure paths explicitly: a sync folder not writable or missing (offline), file already exists, same path with different content, catalog file corrupt, catalog conflict copies present, file missing in one folder.
- Test names with umlauts in both NFC and NFD form, and names that differ only by case.
- Fixtures are **synthetic**. Never copy real documents or real file names of the user into the repository.
- Catalog format tests load real fixture files from `src/test/resources/catalog/`. When the format version changes, keep the old fixture and add a migration test that reads it.
- Round-trip every catalog change: write, read back, compare with `assertEquals`.

## AI tests

- Unit tests never call a real LLM. Test the adapter against recorded responses (fixture JSON) served by a local fake HTTP server or a fake transport.
- Test that malformed, partial, or off-schema model output becomes a typed failure, not an exception and not a half-filled suggestion.
- Integration tests against a running LM Studio are opt-in only (`-Pai.integration=true`) and skipped otherwise.

## UI tests

- Test view models without Compose: feed events, assert the emitted `StateFlow` values.
- Use Compose UI tests (`runComposeUiTest`) only for behavior that lives in the composable itself.
