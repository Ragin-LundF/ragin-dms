//! Every port: what the use cases need from the outside world, by capability. Adapters live in the `infra-*`
//! crates. Port signatures use domain types only; expected failures cross a port as values, never as panics.

pub mod ai;
pub mod assistant;
pub mod export;
pub mod health;
pub mod index;
pub mod mail;
pub mod pdf;
pub mod plan;
pub mod sidecar;
pub mod state;
pub mod storage;
pub mod system;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{ai, assistant, export, health, index, mail, pdf, plan, sidecar, state, storage, system};

    /// Every port must work as `Arc<dyn Port>` shared across threads: object safe, `Send` and `Sync`.
    fn shareable<T: ?Sized + Send + Sync>(_: Option<Arc<T>>) {}

    #[test]
    fn every_port_can_be_shared_between_threads() {
        shareable::<dyn ai::AiServer>(None);
        shareable::<dyn ai::DocumentClassifier>(None);
        shareable::<dyn ai::DocumentContentReader>(None);
        shareable::<dyn ai::DocumentPreviewer>(None);
        shareable::<dyn ai::EmbeddingClient>(None);
        shareable::<dyn ai::PageLayoutReader>(None);
        shareable::<dyn assistant::DocumentAssistant>(None);
        shareable::<dyn assistant::LibraryAssistant>(None);
        shareable::<dyn assistant::LetterWriter>(None);
        shareable::<dyn export::ExportWriter>(None);
        shareable::<dyn health::CatalogSnapshots>(None);
        shareable::<dyn health::HealthStore>(None);
        shareable::<dyn index::VectorIndex>(None);
        shareable::<dyn mail::MailConverter>(None);
        shareable::<dyn pdf::OcrTool>(None);
        shareable::<dyn pdf::PdfEditor>(None);
        shareable::<dyn pdf::PdfRebuilder>(None);
        shareable::<dyn plan::PlanStore>(None);
        shareable::<dyn sidecar::PreviewStore>(None);
        shareable::<dyn sidecar::SidecarLayout>(None);
        shareable::<dyn state::ActivityLog>(None);
        shareable::<dyn state::SettingsRepository>(None);
        shareable::<dyn state::TaskStore>(None);
        shareable::<dyn storage::CatalogRepository>(None);
        shareable::<dyn storage::DocumentStorage>(None);
        shareable::<dyn storage::FolderWatcher>(None);
        shareable::<dyn storage::LibraryScanner>(None);
        shareable::<dyn storage::OperationJournal>(None);
        shareable::<dyn storage::TextStore>(None);
        shareable::<dyn system::Clock>(None);
        shareable::<dyn system::IdGenerator>(None);
    }
}
