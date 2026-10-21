# Coding Guidelines

Use these rules for all Kotlin code unless a more specific instruction file says otherwise.

## Kotlin style

- Use named arguments whenever calling Kotlin functions, constructors, or assertions where named arguments are supported.
- Do not rely on named arguments for Java APIs or APIs where Kotlin named arguments are unavailable.
- Use `runCatching` instead of `try/catch` when it keeps the code correct and readable.
- Use `try/catch` only when `runCatching` would make behavior less correct or less clear, for example with `finally`, resource cleanup, cancellation propagation, or explicit exception flow.
- Never swallow `CancellationException`. `runCatching` around a `suspend` call must rethrow it.
- Implement functions with block bodies. Do not use expression-body functions.
- Keep functions small and focused on a single responsibility.
- Split functions when they become too long, too deeply nested, or mix multiple responsibilities.
- Use descriptive variable, class, and function names.
- Avoid deep nesting. Prefer early returns and guard clauses.
- Avoid files with multiple classes, enums, interfaces, or objects. Use one top-level declaration per file.
- Use meaningful comments for complex logic, non-obvious decisions, trade-offs, or domain rules.
- Do not add comments that merely restate obvious code.
- Keep class and function names consistent with their purpose and behavior.
- Follow the official Kotlin style guide for formatting, naming, and conventions not defined here.

## Design rules

- Classes should follow single responsibility.
- Functions should follow single responsibility.
- Keep public APIs explicit and predictable.
- Prefer immutable data structures and values where practical.
- Preserve existing behavior unless the task explicitly asks for a behavior change.
- Code must be high-performant, clean, readable, and maintainable.
- Be "lazy" when writing code. Don't repeat yourself and don't write unnecessary or bloated code.
- No God classes or methods.
- Avoid code duplication.
- Respect the dependency rule in `architecture-overview.md`. An import that points outwards is a bug, even if it compiles.

## Package layout

- Every module has its own root package: `dms.domain`, `dms.application`, `dms.storage`, `dms.ai`, `dms.ui`.
- Keep at most 8 Kotlin files per directory. When a directory grows past 8, split it into subpackages by responsibility.
- Direct subclasses of a `sealed` type must stay in the same package as the sealed declaration. Never split a sealed hierarchy across packages — Kotlin forbids it.
- An `actual` declaration must stay in the same package as its `expect`.
- Mirror the main package structure in tests. A test moves only when the type it covers moves.

## Models and DTOs

A **model** is a pure data declaration with no behavior: `data class`, `enum class`, `sealed interface` / `sealed class` hierarchies of data, `@JvmInline value class`, and `data object`.

- **`dms-domain`:** the domain *is* the model. Group by subject: `dms.domain.document`, `dms.domain.catalog`, `dms.domain.taxonomy`, `dms.domain.library`. Domain types carry no framework or serialization annotations.
- **`dms-application`:** use case inputs and outcomes live in a `model` subpackage of the use case package, for example `dms.application.document.model`. Ports live in `dms.application.port`.
- **Adapters (`dms-storage`, `dms-ai`):** serialization types live in `dto` packages (`dms.storage.catalog.dto`, `dms.ai.client.dto`) and are mapped to domain types by a mapper in the adapter. A DTO never leaves its adapter.
- **`dms-ui`:** every model and every enum lives in a `model` subpackage of its feature package — `ui.library.model`, `ui.inbox.model`. Never leave a model beside composables or view models.
  - Group inside `model` when it exceeds 8 files.
  - Composables, view models, services, mappers, and objects holding logic never go in a `model` package.
  - Extension functions belong next to the type they extend, including inside a `model` package.
- One top-level declaration per file, named after the declaration.
- Models are immutable. Wrap primitive identifiers in `@JvmInline value class` (`DocumentId`, `CategoryId`, `TagName`, `RelativePath`, `ContentHash`), never pass raw strings where a typed id is expected.
- The catalog file is persisted user data. Changing what it stores is a format change — see `dms-storage.md`.

## Tests

- Use `kotlin.test` annotations and assertions when possible.
- Use named arguments for assertions, for example `assertEquals(expected = expectedValue, actual = actualValue)`.
- Cover relevant code paths, edge cases, and error paths.
- Add or update tests for behavior changes.

## Common rules

- Never commit code. Everything must be reviewed and committed by the user.
- When executing plans, follow the instructions provided in the plan. Ask after every step if the result is correct before continuing, unless it is explicitly unwanted by user prompt.
- Ask always for clarification if needed.
- Never run code against the user's real library folders during development or tests. Use temporary directories.
- The user's documents are private. Never read their content, never use information from them (file names, folder names, structure, counts, conventions) in code, tests, fixtures, prompts or documentation, and never send document content anywhere except to the AI provider the user configured at runtime.
- Any private playground folder the user provides is for manual trying-out only. Test data is always synthetic, created by the tests themselves with invented names.
