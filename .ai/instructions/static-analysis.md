# Static Analysis Instructions

Use this file when fixing clippy, rustfmt or TypeScript findings, changing style-sensitive code, or adding `#[allow]`s.

## Rust: clippy and rustfmt

- The lints live in the root `Cargo.toml` under `[workspace.lints]`: `clippy::pedantic` (warn), `clippy::unwrap_used` (deny), `unsafe_code` (forbid), `module_name_repetitions` allowed. Every crate inherits them with `[lints] workspace = true`; a new crate must do the same.
- All code must pass `cargo clippy --workspace --all-targets -- -D warnings` (CI treats every warning as an error) and `cargo fmt --all --check`.
- Prefer fixing the underlying issue instead of allowing it.
- Use `#[allow(clippy::…)]` only for an individual, justified case, on the smallest item possible, naming the specific lint. Add a short reason when the justification is not obvious.
- Do not allow lints for a whole crate or module. The only exceptions are deliberate: `must_use_candidate` and `missing_panics_doc` in `dms-application`; `needless_pass_by_value` in Tauri command modules (Tauri passes `State` and arguments by value).
- Never weaken the workspace lints or `unsafe_code = "forbid"` to make code compile.
- Words clippy's `doc_markdown` should accept go into the crate's `clippy.toml` (`doc-valid-idents`), not into `#[allow]`s.
- In tests, pedantic flags `assert!(x.is_empty())` on collections where the failure message would be useless; compare with `assert_eq!(x, Vec::<T>::new())` instead.

## TypeScript

- `npm --prefix ui run typecheck` (`tsc --noEmit`, strict mode) must pass; `npm --prefix ui run build` runs it too.
- No `// @ts-ignore` or `any` to silence an error; fix the type. `@ts-expect-error` only with a reason, in tests.
- `ui/src/ipc/bindings.ts` is generated; never edit it by hand. When the Rust commands or DTOs change, regenerate it with `UPDATE_BINDINGS=1 cargo test -p ragin-dms-desktop export_bindings`.
