//! The native menu bar (Kotlin's `AppMenuBar` and `LibraryMenu`) with the main actions and their shortcuts
//! (⌘ on macOS, Ctrl elsewhere). Choosing an entry sends a [`MenuEvent`] to the window, which runs it like the
//! toolbar or the palette would; only "Cancel" is handled here.

use std::sync::{Arc, Mutex};

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Runtime, Wry};
use tauri_specta::Event;

use crate::events::MenuEvent;
use crate::session::{LibraryStatus, Session, SessionState};

/// The entries of the "Library" menu: the palette's library actions, by id, label and shortcut.
const LIBRARY_ACTIONS: [(&str, &str, Option<&str>); 6] = [
    ("scan", "Scan now", Some("CmdOrCtrl+R")),
    ("syncAll", "Sync to all destinations", Some("CmdOrCtrl+Shift+S")),
    ("mergeAll", "Merge all folders", Some("CmdOrCtrl+Shift+M")),
    ("retryFailed", "Retry failed operations", None),
    ("detectDates", "Detect missing document dates", None),
    ("checkArchive", "Check archive now", None),
];

/// The library's keys as menu entries. Only ⌘\ is a real shortcut: one for "/", Space, ⌘A, Esc, ↵ or ⌫ would
/// take that key from the text fields, so those name their key in the label.
const DOCUMENT_ACTIONS: [(&str, &str, Option<&str>); 8] = [
    ("toggleSidebar", "Show or hide the sidebar", Some("CmdOrCtrl+\\")),
    ("focusSearch", "Search documents (/)", None),
    ("quickLook", "Quick look (Space)", None),
    (
        "selectAll",
        if cfg!(target_os = "macos") { "Select all documents (⌘A)" } else { "Select all documents (Ctrl+A)" },
        None,
    ),
    ("clearSelection", "Clear the selection (Esc)", None),
    ("newFolder", "New folder", None),
    ("renameFolder", "Rename the open folder (↵)", None),
    ("deleteFolder", "Delete the open folder… (⌫)", None),
];

const SCREENS: [(&str, &str, &str); 9] = [
    ("openDashboard", "Dashboard", "CmdOrCtrl+1"),
    ("openTasks", "Open tasks", "CmdOrCtrl+2"),
    ("openLibrary", "Library", "CmdOrCtrl+3"),
    ("openBills", "To pay", "CmdOrCtrl+4"),
    ("openContracts", "Contracts", "CmdOrCtrl+5"),
    ("openTax", "Tax years", "CmdOrCtrl+6"),
    ("openCases", "Cases", "CmdOrCtrl+7"),
    ("openEmergency", "Emergency folder", "CmdOrCtrl+8"),
    ("openSettings", "Settings", "CmdOrCtrl+,"),
];

/// Documents entries come after these separators (indexes into [`DOCUMENT_ACTIONS`]).
const DOCUMENT_GROUP_STARTS: [usize; 2] = [3, 5];

/// The entries whose state follows the session and the window.
pub struct MenuItems {
    library: Vec<MenuItem<Wry>>,
    cancel: MenuItem<Wry>,
    documents: Vec<MenuItem<Wry>>,
    show_log: CheckMenuItem<Wry>,
    /// The lane label "Cancel" stops; kept so a click knows what to cancel.
    cancellable: Mutex<Option<crate::session::Lane>>,
}

/// Builds the menu bar and keeps its live entries in the app state.
///
/// # Errors
/// When the platform refuses a menu entry.
pub fn install(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let library: Vec<MenuItem<Wry>> = LIBRARY_ACTIONS
        .iter()
        .map(|(id, label, shortcut)| MenuItem::with_id(app, *id, *label, false, *shortcut))
        .collect::<tauri::Result<_>>()?;
    let cancel = MenuItem::with_id(app, "cancel", "Cancel", false, Some("CmdOrCtrl+."))?;
    let documents: Vec<MenuItem<Wry>> = DOCUMENT_ACTIONS
        .iter()
        .map(|(id, label, shortcut)| MenuItem::with_id(app, *id, *label, false, *shortcut))
        .collect::<tauri::Result<_>>()?;
    let show_log = CheckMenuItem::with_id(app, "showLog", "Show log", true, false, Some("CmdOrCtrl+L"))?;

    let library_menu = Submenu::with_id(app, "library", "Library", true)?;
    for item in &library {
        library_menu.append(item)?;
    }
    library_menu.append(&PredefinedMenuItem::separator(app)?)?;
    library_menu.append(&cancel)?;

    let documents_menu = Submenu::with_id(app, "documents", "Documents", true)?;
    for (index, item) in documents.iter().enumerate() {
        if DOCUMENT_GROUP_STARTS.contains(&index) {
            documents_menu.append(&PredefinedMenuItem::separator(app)?)?;
        }
        documents_menu.append(item)?;
    }

    let view_menu = Submenu::with_id(app, "view", "View", true)?;
    view_menu.append(&MenuItem::with_id(app, "goTo", "Go to…", true, Some("CmdOrCtrl+K"))?)?;
    view_menu.append(&PredefinedMenuItem::separator(app)?)?;
    for (id, label, shortcut) in SCREENS {
        view_menu.append(&MenuItem::with_id(app, id, label, true, Some(shortcut))?)?;
    }
    view_menu.append(&PredefinedMenuItem::separator(app)?)?;
    view_menu.append(&show_log)?;

    // Not in the Kotlin menu: a webview needs the Edit menu for copy and paste on macOS, and the app menu for Quit.
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let menu = Menu::new(app)?;
    if cfg!(target_os = "macos") {
        let app_menu = Submenu::with_items(
            app,
            "Ragin-DMS",
            true,
            &[
                &PredefinedMenuItem::about(app, None, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, None)?,
            ],
        )?;
        menu.append(&app_menu)?;
    }
    menu.append(&library_menu)?;
    menu.append(&edit_menu)?;
    menu.append(&documents_menu)?;
    menu.append(&view_menu)?;
    app.manage(MenuItems { library, cancel, documents, show_log, cancellable: Mutex::new(None) });
    Ok(menu)
}

/// Runs a chosen menu entry: "Cancel" stops the cancellable operation, everything else goes to the window.
pub fn handle(app: &AppHandle, id: &str) {
    if id == "cancel" {
        let lane = app.try_state::<MenuItems>().and_then(|items| *crate::session::lock(&items.cancellable));
        if let (Some(lane), Some(session)) = (lane, app.try_state::<Arc<Session>>()) {
            session.cancel(lane);
        }
        return;
    }
    // The check box toggles itself; the window decides and reports back through `set_menu_context`.
    let _ = MenuEvent { id: id.to_owned() }.emit(app);
}

/// Library actions wait while one runs or no library is open; "Cancel" names what it stops.
pub fn follow_session<R: Runtime>(app: &AppHandle<R>, state: &SessionState) {
    let Some(items) = app.try_state::<MenuItems>() else { return };
    let idle =
        state.busy_in(crate::session::Lane::Library).is_none() && matches!(state.status, LibraryStatus::Open { .. });
    for item in &items.library {
        let _ = item.set_enabled(idle);
    }
    let cancellable = state.operations.iter().find(|operation| operation.cancellable);
    *crate::session::lock(&items.cancellable) = cancellable.map(|operation| operation.lane);
    let label = cancellable.map_or_else(|| "Cancel".to_owned(), |operation| format!("Cancel {}", operation.label));
    let _ = items.cancel.set_text(label);
    let _ = items.cancel.set_enabled(cancellable.is_some());
}

/// The window tells which entries apply: the "Documents" menu while the library screen is shown, and the log.
pub fn set_context<R: Runtime>(app: &AppHandle<R>, library_available: bool, log_visible: bool) {
    let Some(items) = app.try_state::<MenuItems>() else { return };
    for item in &items.documents {
        let _ = item.set_enabled(library_available);
    }
    let _ = items.show_log.set_checked(log_visible);
}
