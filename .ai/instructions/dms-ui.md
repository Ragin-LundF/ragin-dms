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

Feature packages: `ui.dashboard`, `ui.library`, `ui.document` (inspector), `ui.review`, `ui.sync`, `ui.search`, `ui.settings`, `ui.inbox`, `ui.restructure` (the restructure assistant), `ui.ask` (questions to the whole library), `ui.export`, `ui.pdf` (page editor) and `ui.preview` (thumbnails). Each is organised the same way:

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
  - `components.chart`: the dashboard charts (`ColumnChart`, `DonutChart`, `BarList`) with `ChartPointModel`/`ChartSliceModel` in `chart.model`.
  - `components.dialog`: `DialogController`, `DialogHost` and the dialogs.
- `ui.shell` holds the window frame: the brand mark, the one status bar along the bottom (`ActivityBar` with `StatusTexts`: library state, folders online, last scan; running operations with progress, Cancel and Log) and the log panel. `shell.navigation` is the grouped navigation (`Screen.group`, `NavGroup`) with inbox and task badges from `NavigationCounts`; it becomes an icon rail on the Library screen. `shell.toolbar` is the 52 dp window toolbar (title, the field that opens the palette, the screen's actions from `ui.ScreenToolbar`, `SplitButton`). `shell.palette` is the command palette (⌘K / Ctrl+K and View › Go to…): `PaletteItems` lists documents, places and actions, `CommandPalette` holds the query and selection, `PaletteCommands` runs a line through the same view model functions as the buttons. The platform provides `LocalTitleBarInset` (macOS traffic lights) and `LocalCommandKey` ("⌘" or "Ctrl+").
- `ui.session` holds `LibrarySession`, the one owner of the open library: it runs operations through `perform`, one at a time per `OperationLane`: `LIBRARY` for everything that changes files or the catalog, `AI` for the AI's runs, which only add suggestions and so run beside library operations. Screens disable a button only while its own lane is busy (`busy`, `aiBusy`). It keeps the last scan and the folder list, and maps outcomes to messages. View models start work through it, never in parallel to it.
- `ui.service` holds the use case bundles (`AppServices`) that the composition root hands to the view models.
- `ui.review` holds the review of one AI suggestion: `ReviewController` (owned by the dashboard and the inspector view models), `ReviewDialog`, and `ReviewDrafts`, which turns the edited draft into a `SuggestionApprovalModel`. Nothing changes before the user approves. The category field offers every existing category path to pick from and still takes a new name; the suggested file name starts checked only when the AI setting `acceptSuggestedFileNames` is on.
- The library (v2): the sidebar holds views (All documents, Inbox, Needs review, Added this month, Folders, Saved views; no Trash, trashed documents are in the system trash), foldable to a rail and resizable; all filters are `FacetFilter`s in the library's view state, applied only by `LibraryFilter`, shown both as tokens in the search field and as facet chips. Documents show grouped in a grid or list, can be ticked for the bulk bar, and open in one quick look (`document.quicklook.QuickLookController`, shared by the library view model, the folder view and the keys). `library.folders` is the folder view (`FolderBrowser`, `FolderEditor`), `library.picker` the shared "Move to…" picker. Keys are in `library.LibraryKeys`, the menu entries reach the library through `library.LibraryCommands`. Panel widths go through `LibrarySession.layout` only. Fixed component sizes live in `theme.LibrarySizes`.
- The library sidebar folds: each section (`SidebarGroup`) and every folder or category with children has a boxed +/- (`components.FoldToggle`). Subfolders and subcategories start folded; selecting one unfolds its parents. Sidebar entries live in `library.sidebar.item`.
- `ui.assistant` holds what the AI does with one document beside the library's operations, because it only reads: `SummaryController`/`SummaryDialog` and the conversation (`ChatController`, `ChatDialog`). The inspector view model owns both. Tagging and classifying store suggestions and therefore run through `LibrarySession.perform` in the `AI` lane and the review dialog.
- `ui.library.taxonomy.TaxonomyController` creates, renames, moves and deletes categories and tags for the library screen; `ui.document.details.DetailsController` edits fields, note, links and retention.
- `ui.session.NameInput` and `ui.session.DateInput` turn what the user typed into names and dates, with one message each. Use them, and the domain's `parse` functions (`TagNameModel.parse`, `RelativePathModel.parse`), instead of catching exceptions.
- AWT and Swing types stay in `jvmMain`; `commonMain` gets what it needs through `PlatformActions` or a `CompositionLocal` set in `Main` (for example `LocalResizePointerIcon`).
- The `when` over `OpenTask`, an outcome type, `ActivityKind`, `CatalogRejection` or `LibraryStatus` names every case, also in helpers, never `else`.
- `ui.dashboard.AiBatches` runs "Tag with AI" and "Classify with AI" over the library. The dashboard is a one-line `header`, the stat strip (`widgets.StatRow` from `stats.StatColumns`), the `widgets.AiBand`, and `attention` ("Needs your attention": at most 5 rows by severity, each severity also named in words).
- `AppColors.aiGradient` is only for what the AI does (AI band, AI suggestion card, AI buttons) and the logo.
- Dates go through `components.format.DateTexts` (absolute `YYYY-MM-DD`, relative in words with the exact date in a tooltip), sizes through `components.format.FileSizes`.
- `ui.settings`: a configured library's Settings shows one section at a time, chosen from `SettingsSectionList` (`model.SettingsSection`); cards keep unsaved drafts in `rememberSaveable` so they survive a section switch.
- AI is optional per device: `AiSettingsModel.enabled` ("Use AI" in Settings › AI, read as `SessionStateModel.aiEnabled`). While it is off, no AI feature is shown: Ask in the navigation, the AI band and inbox analysis on the dashboard, the inbox AI actions and their settings, the AI actions of the quick look and the bulk bar, "Convert to real PDF", and the AI actions of the toolbar and the palette. A new AI feature must hide the same way.
- `ui.settings.ai` holds the AI settings card: the general switches ("Read PDFs as images", suggested file names), timeout and text size, then the provider chips (`AiProvider`: Local AI or Anthropic, both setups kept) with that provider's fields, and the connection test that lists the server's models. Saving goes through `LibrarySession.updatePreferences`, the one writer of the preferences.
- UI tests run the real `AppContainer` with a `FakeAiServer` (a Ktor `MockEngine`) in place of LM Studio; tests never call a real AI server.
- Confirmations and text input go through `DialogController` in `ui.components.dialog`; a view model asks, a `DialogHost` renders.
- `ui.theme` holds colours, typography, shapes and spacing. The type scale is set in Geist (`AppTypography.withGeist()`), bundled in `composeResources/font` with its licence in `composeResources/files/geist-OFL.txt`; the `Res` class lives in `dms.ui.resources`.
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
- View models expose one immutable `StateFlow<XxxUiState>` per screen and accept events through functions. Composables are "dumb": they render state and emit events via lambdas (`onTagAdded: (TagNameModel) -> Unit`).
- Launch coroutines in `viewModelScope`. Long-running work (scan, sync, batch classification) reports progress through state and can be cancelled; batch classification can also be paused.
- Every persisted setting has **one writer** (a settings controller). Composables never write settings directly.
- Derive, don't duplicate: the selected document, the visible list, the filter result and the task counts are derived from one source of truth, not stored in several places.

## Drag and drop

- **Files from the file manager**: `FileDropZone` accepts a drop, `PlatformActions.droppedFiles` turns the event into paths (only `jvmMain` touches AWT), and the view model adds them like files from the file picker: in the library to the selected folder, on the dashboard to the top level.
- **Inside the library**: a document is dragged with a pointer gesture onto a folder, a category, "Uncategorized" or a tag in the sidebar. `DocumentDrag` keeps the drag state and the registered drop targets; the drop goes to `LibraryViewModel.drop`, which runs the same use cases as the buttons. A drop that changes nothing does nothing. A category of the sidebar is dragged the same way (`DragItem.DraggedCategory`) into another category or onto the "Categories" heading (`DropTarget.TopLevelCategory`) to move it to the top level; `DocumentDrag` only offers targets that take the dragged item, and the catalog refuses cycles.
- Every drop target highlights while a document hovers over it and names what the drop will do.

## Progress and the log

- Long operations report progress through `LibrarySession.progress` (`OperationProgressModel`: step, done of total, current item), kept apart from `SessionStateModel` so screens do not recompute on every file. Loops over many items use `LibrarySession.forEachWithProgress`.
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
- `ui.library.search` searches the documents' text next to the name filter: `ContentSearch` runs `SearchContents` in the background after a typing pause while its switch is on; the matches join the filter in `LibraryFilter.documents(textMatches)`. It only reads, so it never waits for a lane.
