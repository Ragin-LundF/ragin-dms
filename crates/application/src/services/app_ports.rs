use std::sync::Arc;

use crate::port::ai::{
    AiServer, DocumentClassifier, DocumentContentReader, DocumentPreviewer, EmbeddingClient, PageLayoutReader,
};
use crate::port::assistant::{DocumentAssistant, LibraryAssistant};
use crate::port::export::ExportWriter;
use crate::port::index::VectorIndex;
use crate::port::pdf::{OcrTool, PdfEditor, PdfRebuilder};
use crate::port::plan::PlanStore;
use crate::port::sidecar::{PreviewStore, SidecarLayout};
use crate::port::state::{ActivityLog, SettingsRepository, TaskStore};
use crate::port::storage::{
    CatalogRepository, DocumentStorage, FolderWatcher, LibraryScanner, OperationJournal, TextStore,
};
use crate::port::system::{Clock, IdGenerator};

/// One adapter per port; the composition root fills it and hands it to [`super::AppServices::new`]. One adapter
/// may stand behind several fields (the storage adapter usually is `storage`, `scanner`, `catalogs` and `texts`).
#[derive(Clone)]
pub struct AppPorts {
    pub storage: Arc<dyn DocumentStorage>,
    pub scanner: Arc<dyn LibraryScanner>,
    pub catalogs: Arc<dyn CatalogRepository>,
    pub texts: Arc<dyn TextStore>,
    pub plans: Arc<dyn PlanStore>,
    pub index: Arc<dyn VectorIndex>,
    pub previews: Arc<dyn PreviewStore>,
    pub sidecar_layout: Arc<dyn SidecarLayout>,
    pub journal: Arc<dyn OperationJournal>,
    pub watcher: Arc<dyn FolderWatcher>,
    pub settings: Arc<dyn SettingsRepository>,
    pub tasks: Arc<dyn TaskStore>,
    pub activity: Arc<dyn ActivityLog>,
    pub exports: Arc<dyn ExportWriter>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub reader: Arc<dyn DocumentContentReader>,
    pub classifier: Arc<dyn DocumentClassifier>,
    pub ai_server: Arc<dyn AiServer>,
    pub previewer: Arc<dyn DocumentPreviewer>,
    pub assistant: Arc<dyn DocumentAssistant>,
    pub library_assistant: Arc<dyn LibraryAssistant>,
    pub embeddings: Arc<dyn EmbeddingClient>,
    pub layout_reader: Arc<dyn PageLayoutReader>,
    pub pdf_editor: Arc<dyn PdfEditor>,
    pub ocr: Arc<dyn OcrTool>,
    pub rebuilder: Arc<dyn PdfRebuilder>,
}
