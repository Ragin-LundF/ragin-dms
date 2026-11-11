# Coding Guidelines

Use these rules for all Rust and TypeScript code unless a more specific instruction file says otherwise.

## Rust style

- Follow the official Rust style and API guidelines for everything not defined here. `rustfmt` decides formatting (`rustfmt.toml`, width 120).
- Never use `unwrap()`; the workspace denies it, also in tests. Use `?`, a typed error, or `expect("invariant: …")` where a broken invariant is a programming error, and document it under `# Panics`. In tests, `expect("what was expected")` is fine.
- No `unsafe` code; the workspace forbids it. Use a safe crate (for example `rustix`) instead.
- Never swallow an error. `let _ = …`, `.ok()` or an empty `Err(_) => {}` on a fallible call needs a reason in a short comment, and must never hide a failed write of user data.
- Expected failures are values (`Result`, an outcome enum). Error enums derive `thiserror::Error`. Panics are only for broken invariants.
- Match exhaustively on domain enums (`OpenTask`, `DetectedChange`, outcomes, `StorageFailure`, `CatalogRejection`): name every variant, never `_ =>`, so the compiler names each place a new variant must be handled.
- Borrow arguments (`&DocumentId`, `&RelativePath`, `&[T]`, `&str`, `Option<&str>`); take ownership only when the function keeps the value.
- Keep functions small and focused on a single responsibility. Split them when they become too long, too deeply nested, or mix responsibilities.
- Avoid deep nesting. Prefer early returns, `?`, `let … else`, and guard clauses.
- Use descriptive variable, type, and function names, consistent with their behavior.
- One main type per file, the file named after the type in snake_case (`catalog/model/catalog.rs` holds `Catalog`). Small private helpers may stay in the file of the type they serve. Tests live in a `#[cfg(test)] mod tests` (or a `tests.rs` beside the module) or in the crate's `tests/` binary.
- Use doc comments (`///`, `//!`) for public items whose purpose is not obvious from the name. Use comments for complex logic, non-obvious decisions, trade-offs, or domain rules. Do not add comments that restate the code.
- A known shortcut gets a comment `shortcut: <the limit>, <when to upgrade>`.

## Design rules

- Types and functions follow single responsibility. No God types or functions.
- Keep public APIs explicit and predictable; keep items private or `pub(crate)` unless another crate needs them.
- Prefer immutable values. Catalog operations return a new value instead of mutating shared state.
- Preserve existing behavior unless the task explicitly asks for a behavior change.
- Code must be performant, clean, readable, and maintainable. Pick the right data structure (`IndexMap`/`IndexSet` where order matters, as in the 1.x format), do no needless work, and never block the UI thread.
- Be "lazy" when writing code. Don't repeat yourself and don't write unnecessary or bloated code.
- Respect the dependency rule in `architecture-overview.md`. A dependency that points outwards is a bug, even if someone made it compile.

## Module layout

- Every crate has its own root module: `dms_domain`, `dms_application`, `dms_infra_storage`, `dms_infra_ai`, `dms_infra_pdf`, and the desktop binary `ragin-dms-desktop`.
- Keep at most 8 source files per directory. When a directory grows past 8, split it into submodules by responsibility.
- Each module re-exports its main types one level up (`pub use`), so `dms_domain::catalog::Catalog` works next to `dms_domain::catalog::model::Catalog`.
- Mirror the source module structure in test binaries (`crates/<crate>/tests/spec/<module>/`). A test moves only when the type it covers moves.

## Models and DTOs

A **model** is a pure data type without behavior beyond construction, validation and simple queries: structs, enums of data, and newtypes.

- **`dms-domain`:** the domain *is* the model. Group by subject: `dms_domain::document`, `catalog`, `taxonomy`, `library`, …; data types live in the `model` submodule of their subject (`dms_domain::document::model::Document`). Domain types carry no serde or other framework attributes. The allowed exception to "no behavior" is `Catalog`, the aggregate root: its functions are the only way to change documents, categories and tags (see `domain-application.md`).
- **`dms-application`:** use case inputs and outcomes live in the `model` submodule of the use case module (`dms_application::document::model`). Ports live in `dms_application::port::<area>`, their types in `port::<area>::model`.
- **Adapters:** serialization types end in `Dto` and live in a `dto` module of their adapter (`dms_infra_storage::catalog::dto`, `dms_infra_storage::state::dto`, `dms_infra_ai::client::dto`). A mapper in the adapter maps them to domain types. A DTO never leaves its adapter.
- **`src-tauri`:** IPC types end in `Dto` and live in `features/<area>/dto.rs` (or `dto/`). Domain types never cross IPC. DTO names are global in the generated bindings, so prefix them with the area (`LibraryStateDto`, `DashboardButtonDto`).
- Type names carry no `Model` suffix (`Document`, `Catalog`, `RelativePath`, `ClassificationSuggestion`).
- Wrap identifiers and validated values in newtypes with private fields and a checking constructor (`DocumentId`, `CategoryId`, `TagName`, `RelativePath`, `ContentHash`). Never pass raw strings where a typed id is expected. Constructors that can fail return `Result<_, DomainError>`.
- The catalog file is persisted user data. Changing what it stores is a format change — see `infra-storage.md`.

## TypeScript and React

- React 19 function components and hooks, strict TypeScript. No `any`; use the generated types from `ui/src/ipc/bindings.ts`, which is generated, never hand-edited.
- The webview keeps presentation state only (selection, open popovers, drafts, focus, keys). Anything derived from the catalog or the domain rules is computed by a Rust presenter. Never re-implement a domain rule in TypeScript.
- Reads through TanStack Query, actions through plain command calls with their errors reported (`.catch(reportError)`), never ignored.
- Shared components first (`ui/src/shared/components`); a feature's components live in `ui/src/features/<area>`.
- Small components with one job, at most 8 source files per directory, speaking names, no comments that restate code.
- Styles use the theme variables in `ui/src/styles/theme.css`; no hex colours or ad-hoc sizes in components.

## Tests

- Rust: `#[test]` functions with `assert_eq!`/`assert!`/`matches!`; TypeScript: Vitest.
- Cover relevant code paths, edge cases, and error paths. Add or update tests for behavior changes.
- See `testing.md`.

## Common rules

- Never commit, stage, amend, or push. Everything must be reviewed and committed by the user.
- When executing plans, follow the instructions provided in the plan. Ask after every step if the result is correct before continuing, unless it is explicitly unwanted by user prompt.
- Ask always for clarification if needed.
- Never run code or the app against the user's real library folders or the real app data directory during development or tests. Use temporary directories, and `RAGIN_DMS_DATA_DIR` for a scratch app data directory.
- The user's documents are private. Never read their content, never use information from them (file names, folder names, structure, counts, conventions) in code, tests, fixtures, prompts or documentation, and never send document content anywhere except to the AI provider the user configured at runtime.
- Any private playground folder the user provides is for manual trying-out only. Test data is always synthetic, created by the tests themselves with invented names.
