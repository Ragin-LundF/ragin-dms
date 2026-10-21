# UI Instructions

Use this file when touching `dms-ui`, UI source sets, Compose code, view models, or the composition root.

## Platform targets

`dms-ui` is a Kotlin Multiplatform module with a single `jvm()` target (Compose for Desktop) that runs on **macOS, Windows and Linux**, with two source sets:

- `commonMain`: composables, view models, UI state, theme, and `expect` declarations for platform operations (file and folder pickers, drag and drop of files, "show in Finder/Explorer", opening a file in the default app, notifications).
- `jvmMain`: `actual` implementations, the `main` function, and the **composition root**.

Browser and mobile targets are out of scope. Do not add `expect`/`actual` indirection for its own sake.

## Source-set rules

- Put shared composables and UI state in `commonMain`.
- Put only platform-specific I/O or platform API calls in `jvmMain` `actual` implementations. Branch on the OS there, never in `commonMain`.
- `commonMain` depends on `dms-domain` and `dms-application` only. It never imports `dms.storage.*` or `dms.ai.*`.
- The composition root (`dms.ui.AppContainer` in `jvmMain`) is the only place that instantiates adapters and use cases. Use constructor injection; no service locator calls from composables.

## Package layout

Feature packages: `ui.dashboard`, `ui.library`, `ui.document` (inspector), `ui.review`, `ui.sync`, `ui.search`, `ui.settings`. Each is organised the same way:

| Package | Holds |
|---|---|
| `ui.<feature>` | Composables, view models, mappers — anything with behavior. |
| `ui.<feature>.model` | Every model and enum of that feature. See the Models and DTOs rules in `coding-guidelines.md`. |
| `ui.<feature>.<role>` | Further behavior groups when the feature package exceeds 8 files. |
| `ui.<feature>.model.<group>` | Model groups when `model` itself exceeds 8 files. |

- A new data class or enum goes in `model`, never beside the composable that uses it.
- `ui.components` holds only cross-feature Compose widgets — nothing feature-specific.
- `ui.theme` holds colours, typography, shapes and spacing.
- Keep at most 8 files per directory.

## The dashboard

The dashboard is the home screen and the **only place where detected changes become actions**.

- It shows the library folders with their status, the open tasks (`OpenTask` from `BuildDashboard`), bulk actions (categorize all new, sync to all destinations, merge all folders) and recent activity.
- Every task row offers the actions its use case supports, and nothing else. The `when` over `OpenTask` is exhaustive, without `else`.
- An AI suggestion is approved field by field or as a whole, and every value is editable before approval. There is no "apply automatically" path.
- Bulk approval is always an explicit click on a selection the user can see.
- A task disappears only when its use case succeeded or the user dismissed it.

## State management and unidirectional data flow

- UI state is held in view models in `commonMain` (`androidx.lifecycle.ViewModel` from the Compose Multiplatform lifecycle library).
- View models expose one immutable `StateFlow<XxxUiState>` per screen and accept events through functions. Composables are "dumb": they render state and emit events via lambdas (`onTagAdded: (TagName) -> Unit`).
- Launch coroutines in `viewModelScope`. Long-running work (scan, sync, batch classification) reports progress through state and can be cancelled; batch classification can also be paused.
- Every persisted setting has **one writer** (a settings controller). Composables never write settings directly.
- Derive, don't duplicate: the selected document, the visible list, the filter result and the task counts are derived from one source of truth, not stored in several places.

## Errors and feedback

- Use case outcomes are data. Map every `sealed` outcome to UI state; a `when` over an outcome must be exhaustive, without `else`.
- **A refused action says why.** A disabled button carries a tooltip with the reason; a refused drop shows the reason in the status bar. A silent refusal is indistinguishable from a bug.
- Destructive actions (trash, delete everywhere, keeping one side of a conflict) always ask for confirmation and name the affected files and folders.
- Unrecoverable errors (primary folder missing, catalog unreadable) are shown in a dedicated error state with a way to fix them (choose the folder again, open the conflict copy), never as a crash.
- An offline sync folder is a status, not an error.

## UI design system

- Material 3, light and dark theme following the system setting.
- Colours, typography, spacing and shapes are defined in `ui.theme`. Use the named theme values; do not use hardcoded `Color(0x...)` literals or ad-hoc `dp` values for spacing.
- Short labels (tags, chips, file sizes, folder names) use `softWrap = false` and truncate with an ellipsis instead of wrapping inside the word.
- Keyboard first: every main action has a shortcut and a menu entry; the review screen supports approve → next without the mouse. Use the platform's modifier key (⌘ on macOS, Ctrl elsewhere).
- Platform wording comes from one place: "Show in Finder" on macOS, "Show in Explorer" on Windows, "Show in file manager" on Linux.
