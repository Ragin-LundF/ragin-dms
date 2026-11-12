//! The composition root (Kotlin `AppContainer`): the adapters of this device, wired into [`AppServices`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dms_application::port::state::{AiSettings, SettingsRepository};
use dms_application::services::{AppPorts, AppServices};
use dms_infra_ai::AiAdapters;
use dms_infra_pdf::PdfEngine;
use dms_infra_pdf::adapter::PdfAdapters;
use dms_infra_storage::{PlatformTrash, StorageAdapters, SystemTrash};

use super::system::{RandomIds, SystemClock};

/// Builds every use case on the adapters of `data_dir`, with `PDFium` loaded from `pdfium` (a folder or the
/// library file itself).
///
/// # Errors
/// When `PDFium` cannot be loaded; the text names the folder searched.
pub fn compose(data_dir: &Path, pdfium: &Path) -> Result<AppServices, String> {
    Ok(AppServices::new(ports(data_dir, pdfium, Arc::new(PlatformTrash))?))
}

/// The adapters of this device, with `trash` for everything the library moves to the trash.
///
/// # Errors
/// When `PDFium` cannot be loaded.
pub fn ports(data_dir: &Path, pdfium: &Path, trash: Arc<dyn SystemTrash>) -> Result<AppPorts, String> {
    let storage = StorageAdapters::with_trash(data_dir, trash);
    let settings: Arc<dyn SettingsRepository> = storage.settings.clone();
    // The AI adapters read the settings for every request, so a change in Settings applies to the next one.
    let ai_settings = {
        let settings = Arc::clone(&settings);
        Arc::new(move || {
            settings.load_preferences().map_or_else(|_| AiSettings::default(), |preferences| preferences.ai)
        })
    };
    let engine = PdfEngine::bind(pdfium).map_err(|error| format!("PDFium could not be loaded: {error}"))?;
    let pdf = PdfAdapters::new(engine, data_dir.join("rebuild"), ai_settings.clone());
    let ai = AiAdapters::new(ai_settings);
    let client = Arc::new(ai.client);
    let ports = AppPorts {
        storage: storage.documents,
        scanner: storage.scanner,
        catalogs: storage.catalogs,
        texts: storage.texts,
        plans: storage.plans,
        index: storage.index,
        previews: storage.previews,
        sidecar_layout: storage.sidecar_layout,
        journal: storage.journal,
        watcher: storage.watcher,
        settings,
        tasks: storage.tasks,
        activity: storage.activity,
        exports: storage.exports,
        clock: Arc::new(SystemClock::new()),
        ids: Arc::new(RandomIds),
        reader: Arc::new(pdf.reader),
        classifier: client.clone(),
        ai_server: client,
        previewer: Arc::new(pdf.previewer),
        assistant: Arc::new(ai.assistant),
        library_assistant: Arc::new(ai.library_assistant),
        embeddings: Arc::new(ai.embeddings),
        layout_reader: Arc::new(ai.layout_reader),
        pdf_editor: Arc::new(pdf.editor),
        ocr: Arc::new(pdf.ocr),
        rebuilder: Arc::new(pdf.rebuilder),
        letter_writer: Arc::new(ai.letter_writer),
        contract_terms_reader: Arc::new(ai.contract_terms_reader),
        health: storage.health,
        snapshots: storage.snapshots,
        mail: Arc::new(pdf.mail),
        split_proposer: Arc::new(ai.split_proposer),
        query_translator: Arc::new(ai.query_translator),
    };
    Ok(ports)
}

/// Where `PDFium` is: in a macOS bundle the library in `Contents/Frameworks` (signed with the app, so the
/// hardened runtime loads it), elsewhere the resource folder `pdfium/<platform>`, else the development copy fetched
/// by `scripts/fetch-pdfium.sh`.
pub fn pdfium_location(resource_dir: Option<PathBuf>) -> Option<PathBuf> {
    let bundled = resource_dir.and_then(|dir| {
        let framework = dir.parent().map(|contents| contents.join("Frameworks").join("libpdfium.dylib"));
        framework.filter(|file| file.is_file()).or_else(|| Some(dir.join("pdfium").join(platform_folder())))
    });
    bundled.filter(|path| path.exists()).or_else(PdfEngine::dev_library)
}

fn platform_folder() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "mac-x64",
        ("windows", _) => "win-x64",
        ("linux", _) => "linux-x64",
        _ => "mac-arm64",
    }
}
