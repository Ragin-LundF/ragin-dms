# UI Instructions

Use this file when touching `dms-ui`, UI source sets, Compose code, view models, or the composition root.

## Platform targets

`dms-ui` is a Kotlin Multiplatform module with a single `jvm()` target (Compose for Desktop) that runs on **macOS, Windows and Linux**, with two source sets:

- `commonMain`: composables, view models, UI state, theme, and the `PlatformActions` interface for platform operations (file and folder pickers, drag and drop of files, "show in Finder/Explorer", opening a file in the default app, notifications).
- `jvmMain`: `DesktopPlatformActions`, the `main` function, the menu bar, and the **composition root**.

Browser and mobile targets are out of scope. Do not add `expect`/`actual` indirection for its own sake.

## Source-set rules

- Put shared composables and UI state in `commonMain`.
- Put only platform-specific I/O or platform API calls in `jvmMain` (`DesktopPlatformActions`, `HostOs`). Branch on the OS there, never in `commonMain`.
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
- `ui.components` holds only cross-feature Compose widgets — nothing feature-specific. Subpackages:
  - `components.surface`: cards (`AppCard`, `CardTitle`), badges, chips, empty states and the per-kind styles (`KindStyles`).
  - `components.layout`: `ScreenHeader`, `SearchField`, and `FileDropZone` (files dropped from the file manager).
  - `components.chart`: the dashboard charts (`ColumnChart`, `DonutChart`, `BarList`) with `ChartPoint`/`ChartSlice` in `chart.model`.
  - `components.dialog`: `DialogController`, `DialogHost` and the dialogs.
- `ui.shell` holds the window frame: the navigation sidebar, the brand mark, the status footer, the activity bar (progress and Cancel) and the log panel.
- `ui.session` holds `LibrarySession`, the one owner of the open library: it runs one operation at a time (`perform`), keeps the last scan and the folder list, and maps outcomes to messages. View models start work through it, never in parallel to it.
- `ui.service` holds the use case bundles (`AppServices`) that the composition root hands to the view models.
- Confirmations and text input go through `DialogController` in `ui.components.dialog`; a view model asks, a `DialogHost` renders.
- `ui.theme` holds colours, typography, shapes and spacing.
- Keep at most 8 files per directory.

## The dashboard

The dashboard is the home screen and the **only place where detected changes become actions**.

- It shows the library folders with their status, the open tasks (`OpenTask` from `BuildDashboard`), bulk actions (categorize all new, sync to all destinations, merge all folders) and recent activity.
- It is also the entry point into the library: quick access by category and tag (a click opens the library filtered by it), recent documents, and statistics (documents per year, kind, category and correspondent). The figures come from `LibraryStatistics`, a pure function of the catalog; composables never count documents themselves.
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

## Drag and drop

- **Files from the file manager**: `FileDropZone` accepts a drop, `PlatformActions.droppedFiles` turns the event into paths (only `jvmMain` touches AWT), and the view model adds them like files from the file picker: in the library to the selected folder, on the dashboard to the top level.
- **Inside the library**: a document is dragged with a pointer gesture onto a folder, a category, "Uncategorized" or a tag in the sidebar. `DocumentDrag` keeps the drag state and the registered drop targets; the drop goes to `LibraryViewModel.drop`, which runs the same use cases as the buttons. A drop that changes nothing does nothing.
- Every drop target highlights while a document hovers over it and names what the drop will do.

## Progress and the log

- Long operations report progress through `LibrarySession.progress` (`OperationProgress`: step, done of total, current item), kept apart from `SessionState` so screens do not recompute on every file. Loops over many items use `LibrarySession.forEachWithProgress`.
- Only operations that leave the library consistent wherever they stop are `cancellable` (a scan, adopting many files). Cancel is offered in the activity bar and the menu.
- `SessionLog` keeps the last lines of what the app did in this session, for the log panel. It is not written to disk.
- A scan with the preference "Add new files while scanning" catalogs new files of the primary in batches while it runs, so the library fills up live; files only the sync folders have are adopted right after the scan.

## Errors and feedback

- Use case outcomes are data. Map every `sealed` outcome to UI state; a `when` over an outcome must be exhaustive, without `else`.
- **A refused action says why.** A disabled button carries a tooltip with the reason; a refused drop shows the reason in the status bar. A silent refusal is indistinguishable from a bug.
- Destructive actions (trash, delete everywhere, keeping one side of a conflict) always ask for confirmation and name the affected files and folders.
- Unrecoverable errors (primary folder missing, catalog unreadable) are shown in a dedicated error state with a way to fix them (choose the folder again, open the conflict copy), never as a crash.
- An offline sync folder is a status, not an error.

## UI design system

- Material 3, light and dark theme following the system setting. Icons come from `material-icons-extended`.
- The logo's source is `dms-ui/packaging/icons/logo.svg`; the PNG, ICNS and ICO files next to it and `jvmMain/resources/app-icon.png` are rendered from it. `jvmMain` provides it to the UI through `LocalAppLogo`.
- Charts follow fixed rules: a fixed colour order from `ChartColors` (never cycled), thin marks, a legend with counts, and labels only where they help. Status colours (`statusGood`, `statusWarning`, `error`) are reserved for states and never used as a chart colour; a state also carries an icon or a label, never colour alone.
- Colours, typography, spacing and shapes are defined in `ui.theme`. Use the named theme values; do not use hardcoded `Color(0x...)` literals or ad-hoc `dp` values for spacing.
- Short labels (tags, chips, file sizes, folder names) use `softWrap = false` and truncate with an ellipsis instead of wrapping inside the word.
- Keyboard first: every main action has a shortcut and a menu entry; the review screen supports approve → next without the mouse. Use the platform's modifier key (⌘ on macOS, Ctrl elsewhere).
- Platform wording comes from one place: "Show in Finder" on macOS, "Show in Explorer" on Windows, "Show in file manager" on Linux.
