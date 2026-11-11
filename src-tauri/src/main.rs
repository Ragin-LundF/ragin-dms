//! Ragin-DMS desktop app: the composition root, the Tauri commands and the session that runs operations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod context;
mod crash;
mod dto;
#[cfg(test)]
mod e2e_tests;
mod events;
mod features;
mod menu;
mod messages;
mod protocol;
mod session;

use std::sync::Arc;

use tauri::Manager;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};

use crate::session::Session;

fn main() {
    crash::install();
    let specta = specta_builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta.invoke_handler())
        .register_asynchronous_uri_scheme_protocol(protocol::SCHEME, protocol::handle)
        .menu(menu::install)
        .on_menu_event(|app, event| menu::handle(app, event.id().as_ref()))
        .setup(move |app| {
            specta.mount_events(app);
            let events = Arc::new(events::TauriSessionEvents::new(app.handle().clone()));
            let session = Session::new(events);
            app.manage(Arc::clone(&session));
            let ctx = Arc::new(compose(app, Arc::clone(&session)));
            app.manage(Arc::clone(&ctx));
            ctx.follow_library();
            if ctx.services().is_some() {
                // The window shows "Opening the library" meanwhile; a failure becomes the library status.
                let _ = ctx.open();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to start Ragin-DMS");
}

/// The context with the services of this device; without them (no app data folder, `PDFium` missing) the library
/// status says why.
fn compose(app: &tauri::App, session: Arc<Session>) -> context::AppContext {
    let pdfium = context::pdfium_location(app.path().resource_dir().ok());
    let composed = match (context::data_dir(), pdfium) {
        (Some(data_dir), Some(pdfium)) => context::compose(&data_dir, &pdfium),
        (None, _) => Err("the app data folder is unknown on this system".to_owned()),
        (_, None) => Err("PDFium is missing from the app".to_owned()),
    };
    match composed {
        Ok(services) => context::AppContext::with_services(session, services),
        Err(detail) => {
            session.error(format!("The app could not start: {detail}"));
            session.update(|state| state.status = session::LibraryStatus::Failed { detail: Some(detail) });
            context::AppContext::new(session)
        }
    }
}

#[allow(clippy::too_many_lines)] // the list of commands and events, one per line
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Result)
        .commands(collect_commands![
            // shell
            commands::session::session_state,
            commands::session::session_progress,
            commands::session::log_entries,
            commands::session::clear_log,
            commands::session::cancel_operation,
            commands::session::set_menu_context,
            commands::session::file_manager_name,
            commands::session::number_locale,
            commands::operations::open_library,
            commands::operations::scan_library,
            commands::operations::sync_to_all_destinations,
            commands::operations::merge_all_folders,
            commands::operations::retry_failed_operations,
            commands::operations::detect_document_dates,
            commands::operations::navigation_counts,
            commands::operations::analyze_inbox_documents,
            commands::operations::set_add_new_files_while_scanning,
            commands::palette::palette_matches,
            // features::dashboard (dashboard + open tasks): add commands below this line
            features::dashboard::commands::dashboard_view,
            features::dashboard::commands::run_dashboard_action,
            features::dashboard::commands::ai_tag_all,
            features::dashboard::commands::ai_classify_all,
            features::dashboard::commands::find_scans_without_text,
            features::dashboard::commands::export_calendar,
            features::dashboard::commands::open_task_file,
            // features::library (library screen, folders, export): add commands below this line
            features::library::commands::view::library_state,
            features::library::commands::view::library_disk_folders,
            features::library::commands::view::library_folder_view,
            features::library::commands::view::library_folder_tree,
            features::library::commands::view::library_proposed_folder_name,
            features::library::commands::view::library_layout,
            features::library::commands::view::library_set_layout,
            features::library::commands::bulk::library_bulk_assign_category,
            features::library::commands::bulk::library_bulk_add_tag,
            features::library::commands::bulk::library_bulk_move,
            features::library::commands::bulk::library_bulk_trash,
            features::library::commands::bulk::library_bulk_tag_with_ai,
            features::library::commands::taxonomy::library_add_category,
            features::library::commands::taxonomy::library_rename_category,
            features::library::commands::taxonomy::library_remove_category,
            features::library::commands::taxonomy::library_rename_tag,
            features::library::commands::taxonomy::library_remove_tag_request,
            features::library::commands::taxonomy::library_remove_tag,
            features::library::commands::saved_views::library_save_view,
            features::library::commands::saved_views::library_replace_saved_view,
            features::library::commands::saved_views::library_rename_saved_view,
            features::library::commands::saved_views::library_delete_saved_view,
            features::library::commands::saved_views::library_add_standard_saved_views,
            features::library::commands::folders::library_create_folder,
            features::library::commands::folders::library_rename_folder,
            features::library::commands::folders::library_move_folder_into,
            features::library::commands::folders::library_folder_delete_request,
            features::library::commands::folders::library_trash_folder,
            features::library::commands::folders::library_add_files,
            features::library::commands::export::library_export,
            // features::document (quick look, details, review, assistant): add commands below this line
            features::document::commands::quick_look::quick_look,
            features::document::commands::quick_look::save_quick_fact,
            features::document::commands::quick_look::assign_document_category,
            features::document::commands::quick_look::remove_document_tag,
            features::document::commands::quick_look::open_document_file,
            features::document::commands::quick_look::show_document_in_file_manager,
            features::document::commands::quick_look::rename_document,
            features::document::commands::quick_look::move_document_to_folder,
            features::document::commands::quick_look::trash_question,
            features::document::commands::quick_look::move_document_to_trash,
            features::document::commands::quick_look::classify_document_with_ai,
            features::document::commands::quick_look::tag_document_with_ai,
            features::document::commands::review::suggestion_review,
            features::document::commands::review::approve_review,
            features::document::commands::review::accept_suggestion,
            features::document::commands::review::reject_review,
            features::document::commands::review::dismiss_suggestion,
            features::document::commands::details::add_standard_fields,
            features::document::commands::details::save_document_fields,
            features::document::commands::details::save_document_note,
            features::document::commands::details::save_keep_until,
            features::document::commands::details::save_category_years,
            features::document::commands::details::create_rule_from_document,
            features::document::commands::details::link_candidates_of,
            features::document::commands::details::link_documents,
            features::document::commands::details::unlink_documents,
            features::document::commands::details::related_documents_of,
            features::document::commands::assistant::summarize_document,
            features::document::commands::assistant::ask_about_document,
            // features::settings and features::inbox: add commands below this line
            features::settings::commands::settings_state,
            features::settings::commands::library_settings,
            features::settings::commands::configure_library,
            features::settings::commands::save_add_new_files_while_scanning,
            features::settings::commands::save_ai_settings,
            features::settings::commands::check_ai_draft,
            features::settings::commands::test_ai_connection,
            features::settings::commands::change_library_settings,
            features::settings::commands::check_inbox_path,
            features::settings::commands::save_inbox,
            features::inbox::commands::inbox_state,
            features::inbox::commands::analyze_inbox_document,
            features::inbox::commands::analyze_inbox,
            features::inbox::commands::file_inbox_document,
            // features::restructure, features::ask and features::pdf: add commands below this line
            features::restructure::commands::restructure_state,
            features::restructure::commands::restructure_history,
            features::restructure::commands::preview_restructure,
            features::restructure::commands::restructure_apply_preview,
            features::restructure::commands::apply_restructure,
            features::restructure::commands::reverse_restructure,
            features::restructure::commands::remove_empty_folders,
            features::restructure::commands::dismiss_empty_folders,
            features::restructure::commands::save_path_template,
            features::ask::commands::ask_state,
            features::ask::commands::ask_library,
            features::ask::commands::update_ask_index,
            features::pdf::commands::pdf_tools,
            features::pdf::commands::make_searchable,
            features::pdf::commands::make_all_searchable,
            features::pdf::commands::merge_start,
            features::pdf::commands::merge_matches,
            features::pdf::commands::merge_documents,
            features::pdf::commands::pdf_page_count,
            features::pdf::commands::edit_pdf_pages,
            features::pdf::rebuild_commands::start_rebuild,
            features::pdf::rebuild_commands::rebuild_compare,
            features::pdf::rebuild_commands::choose_rebuild,
        ])
        .events(collect_events![
            events::SessionStateEvent,
            events::ProgressEvent,
            events::LogEvent,
            events::MessageEvent,
            events::MenuEvent,
            // feature events: add below this line
            features::document::dto::OpenReviewEvent,
        ])
}

#[cfg(test)]
mod tests {
    const BINDINGS: &str = "../ui/src/ipc/bindings.ts";

    /// Fails when `ui/src/ipc/bindings.ts` is out of date; `UPDATE_BINDINGS=1` rewrites it.
    /// A changed DTO then breaks the TypeScript build, not the running app.
    #[test]
    fn export_bindings() {
        let fresh = std::env::temp_dir().join(format!("ragin-dms-bindings-{}.ts", std::process::id()));
        super::specta_builder()
            .export(specta_typescript::Typescript::default(), &fresh)
            .expect("export TypeScript bindings");
        let generated = std::fs::read_to_string(&fresh).expect("read generated bindings");
        let _ = std::fs::remove_file(&fresh);
        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            std::fs::write(BINDINGS, generated).expect("write bindings");
            return;
        }
        let current = std::fs::read_to_string(BINDINGS).unwrap_or_default();
        // Whitespace is ignored: an IDE that reformats the file on commit (import spacing) does not make it stale.
        let without_whitespace = |text: &str| text.split_whitespace().collect::<String>();
        assert!(
            without_whitespace(&current) == without_whitespace(&generated),
            "{BINDINGS} is stale: run `UPDATE_BINDINGS=1 cargo test -p ragin-dms-desktop export_bindings`"
        );
    }
}
