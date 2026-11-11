use std::sync::Arc;

use crate::ask::{AskLibrary, DocumentTexts, IndexLibrary, RelatedDocuments, VectorSearch};
use crate::assistant::{AskAboutDocument, DocumentReading, SuggestTags, SummarizeDocument, TagAllUntagged};
use crate::catalog::{ChangeSettings, EditCatalog};
use crate::classification::{ApproveSuggestion, ClassifyAllNew, ListAiModels, RejectSuggestion, SuggestClassification};
use crate::common::{ActivityRecorder, DocumentDates, DocumentFiling, SiblingNames};
use crate::deadline::SettleRetention;
use crate::document::{
    AddDocuments, AdoptNewFile, EditDocumentDetails, MoveDocument, RestoreDocument, TrashDocument, TrashDuplicateFile,
    TrashNewFile,
};
use crate::export::ExportDocuments;
use crate::folder::{CreateFolder, MoveFolder, ReadFolderEntries, ReadFolders, TrashFolder};
use crate::inbox::{CollectInbox, ConfigureInbox, FileInboxDocument};
use crate::library::{
    AddScannedFiles, BuildDashboard, CatalogStore, ConfigureLibrary, DetectDocumentDates, OpenLibrary, ScanLibrary,
};
use crate::pdf::rebuild::{KeepRebuild, RebuildAvailability, RebuildPdf};
use crate::pdf::{
    EditPages, EmbedPdfMetadata, MakeAllSearchable, MakeSearchable, MergeDocuments, PdfVersions, ScansWithoutText,
    SplitDocument,
};
use crate::preview::DocumentThumbnails;
use crate::resolution::{DismissTask, KeepBothVersions, KeepVersion, MergeDuplicates, ResolveMove};
use crate::restructure::{
    ApplyRestructurePlan, ListRestructureHistory, PlanRestructure, RemoveEmptyFolders, ReverseRestructure,
};
use crate::rule::AcceptRuleProposal;
use crate::search::{ContentMatcher, FindDocuments, SearchContents};
use crate::services::{
    AiServices, AppPorts, AskServices, DocumentServices, LibraryServices, OrganizeServices, PdfServices, SyncServices,
};
use crate::sidecar::MigrateSidecarLayout;
use crate::sync::{MergeAllFolders, Replicator, RetryFailedOperations, SyncFromPrimary};

/// Every use case the UI calls, wired once from the adapters. Stateful parts (the catalog store, the content
/// matcher, the caches of thumbnails, vector search, document texts, the document assistant and the scan-text
/// check) exist exactly once and are shared by every use case that needs them.
pub struct AppServices {
    pub library: LibraryServices,
    pub documents: DocumentServices,
    pub sync: SyncServices,
    pub ai: AiServices,
    pub organize: OrganizeServices,
    pub pdf: PdfServices,
    pub ask: AskServices,
}

/// The parts several bundles share, as in Kotlin's `AppContainer`.
struct Core {
    ports: AppPorts,
    replicator: Arc<Replicator>,
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
    names: Arc<SiblingNames>,
    dates: Arc<DocumentDates>,
    matcher: Arc<ContentMatcher>,
    trash_document: Arc<TrashDocument>,
    move_document: Arc<MoveDocument>,
    change_settings: Arc<ChangeSettings>,
    filing: Arc<DocumentFiling>,
    adopt_new_file: Arc<AdoptNewFile>,
    open_library: Arc<OpenLibrary>,
}

impl AppServices {
    pub fn new(ports: AppPorts) -> Self {
        let core = Core::new(ports);
        Self {
            library: core.library_services(),
            documents: core.document_services(),
            sync: core.sync_services(),
            ai: core.ai_services(),
            organize: core.organize_services(),
            pdf: core.pdf_services(),
            ask: core.ask_services(),
        }
    }
}

impl Core {
    fn new(ports: AppPorts) -> Self {
        let p = &ports;
        let replicator = Arc::new(Replicator::new(
            p.storage.clone(),
            p.catalogs.clone(),
            p.texts.clone(),
            p.plans.clone(),
            p.index.clone(),
            p.journal.clone(),
            p.clock.clone(),
            p.ids.clone(),
        ));
        let store = Arc::new(CatalogStore::new(p.settings.clone(), p.catalogs.clone(), replicator.clone()));
        let activity = Arc::new(ActivityRecorder::new(p.activity.clone(), p.clock.clone()));
        let names = Arc::new(SiblingNames::new(p.storage.clone()));
        let dates = Arc::new(DocumentDates::new(p.storage.clone()));
        let matcher = Arc::new(ContentMatcher::new(p.reader.clone(), p.texts.clone(), p.clock.clone()));
        let trash_document =
            Arc::new(TrashDocument::new(store.clone(), replicator.clone(), p.tasks.clone(), activity.clone()));
        let move_document = Arc::new(MoveDocument::new(
            store.clone(),
            p.storage.clone(),
            replicator.clone(),
            names.clone(),
            activity.clone(),
        ));
        let change_settings = Arc::new(ChangeSettings::new(store.clone(), activity.clone()));
        let filing = Arc::new(DocumentFiling::new(store.clone(), names.clone(), move_document.clone()));
        let adopt_new_file = Arc::new(AdoptNewFile::new(
            store.clone(),
            p.storage.clone(),
            dates.clone(),
            replicator.clone(),
            activity.clone(),
            p.clock.clone(),
            p.ids.clone(),
        ));
        let open_library = Arc::new(OpenLibrary::new(
            p.settings.clone(),
            p.catalogs.clone(),
            replicator.clone(),
            store.clone(),
            p.tasks.clone(),
            p.ids.clone(),
        ));
        Self {
            ports,
            replicator,
            store,
            activity,
            names,
            dates,
            matcher,
            trash_document,
            move_document,
            change_settings,
            filing,
            adopt_new_file,
            open_library,
        }
    }

    fn library_services(&self) -> LibraryServices {
        let p = &self.ports;
        let add_scanned_files = Arc::new(AddScannedFiles::new(
            self.store.clone(),
            self.dates.clone(),
            self.activity.clone(),
            p.clock.clone(),
            p.ids.clone(),
        ));
        LibraryServices {
            store: self.store.clone(),
            settings: p.settings.clone(),
            clock: p.clock.clone(),
            configure_library: Arc::new(ConfigureLibrary::new(p.settings.clone(), p.ids.clone())),
            open_library: self.open_library.clone(),
            migrate_sidecar_layout: Arc::new(MigrateSidecarLayout::new(self.store.clone(), p.sidecar_layout.clone())),
            scan_library: Arc::new(ScanLibrary::new(self.store.clone(), p.scanner.clone(), add_scanned_files)),
            detect_document_dates: Arc::new(DetectDocumentDates::new(
                self.store.clone(),
                self.dates.clone(),
                self.activity.clone(),
            )),
            build_dashboard: Arc::new(BuildDashboard::new(
                self.store.clone(),
                p.tasks.clone(),
                p.journal.clone(),
                p.activity.clone(),
                p.clock.clone(),
            )),
            search_contents: Arc::new(SearchContents::new(self.store.clone(), self.matcher.clone())),
            find_documents: Arc::new(FindDocuments::new(self.store.clone(), self.matcher.clone())),
            thumbnails: Arc::new(DocumentThumbnails::new(self.store.clone(), p.previewer.clone(), p.previews.clone())),
        }
    }

    fn document_services(&self) -> DocumentServices {
        let p = &self.ports;
        let (store, activity) = (&self.store, &self.activity);
        DocumentServices {
            add_documents: Arc::new(AddDocuments::new(
                store.clone(),
                p.storage.clone(),
                self.replicator.clone(),
                self.names.clone(),
                activity.clone(),
                p.clock.clone(),
                p.ids.clone(),
            )),
            adopt_new_file: self.adopt_new_file.clone(),
            move_document: self.move_document.clone(),
            trash_document: self.trash_document.clone(),
            trash_new_file: Arc::new(TrashNewFile::new(store.clone(), p.storage.clone(), activity.clone())),
            trash_duplicate_file: Arc::new(TrashDuplicateFile::new(store.clone(), p.storage.clone(), activity.clone())),
            restore_document: Arc::new(RestoreDocument::new(
                store.clone(),
                p.storage.clone(),
                self.replicator.clone(),
                activity.clone(),
            )),
            create_folder: Arc::new(CreateFolder::new(
                store.clone(),
                p.storage.clone(),
                self.replicator.clone(),
                self.names.clone(),
                activity.clone(),
            )),
            move_folder: Arc::new(MoveFolder::new(
                store.clone(),
                p.storage.clone(),
                self.replicator.clone(),
                self.names.clone(),
                activity.clone(),
            )),
            read_folder_entries: Arc::new(ReadFolderEntries::new(store.clone(), p.storage.clone())),
            read_folders: Arc::new(ReadFolders::new(store.clone(), p.storage.clone())),
            trash_folder: Arc::new(TrashFolder::new(
                store.clone(),
                self.replicator.clone(),
                p.tasks.clone(),
                activity.clone(),
            )),
            edit_catalog: Arc::new(EditCatalog::new(store.clone(), activity.clone(), p.ids.clone())),
            change_settings: self.change_settings.clone(),
            edit_document_details: Arc::new(EditDocumentDetails::new(store.clone(), activity.clone())),
            settle_retention: Arc::new(SettleRetention::new(store.clone(), activity.clone(), p.clock.clone())),
        }
    }

    fn sync_services(&self) -> SyncServices {
        let p = &self.ports;
        let (store, activity, replicator) = (&self.store, &self.activity, &self.replicator);
        SyncServices {
            sync_from_primary: Arc::new(SyncFromPrimary::new(
                store.clone(),
                p.scanner.clone(),
                replicator.clone(),
                activity.clone(),
            )),
            merge_all_folders: Arc::new(MergeAllFolders::new(
                store.clone(),
                p.scanner.clone(),
                p.storage.clone(),
                replicator.clone(),
                self.open_library.clone(),
                activity.clone(),
            )),
            retry_failed_operations: Arc::new(RetryFailedOperations::new(
                store.clone(),
                p.journal.clone(),
                replicator.clone(),
            )),
            keep_version: Arc::new(KeepVersion::new(
                store.clone(),
                p.storage.clone(),
                replicator.clone(),
                activity.clone(),
            )),
            keep_both_versions: Arc::new(KeepBothVersions::new(
                store.clone(),
                p.storage.clone(),
                replicator.clone(),
                self.names.clone(),
                activity.clone(),
            )),
            resolve_move: Arc::new(ResolveMove::new(
                store.clone(),
                p.storage.clone(),
                replicator.clone(),
                activity.clone(),
            )),
            dismiss_task: Arc::new(DismissTask::new(p.tasks.clone())),
            merge_duplicates: Arc::new(MergeDuplicates::new(
                store.clone(),
                p.storage.clone(),
                p.tasks.clone(),
                activity.clone(),
            )),
        }
    }

    fn ai_services(&self) -> AiServices {
        let p = &self.ports;
        let suggest = Arc::new(SuggestClassification::new(
            self.store.clone(),
            p.reader.clone(),
            p.classifier.clone(),
            p.tasks.clone(),
            p.texts.clone(),
            self.replicator.clone(),
            p.clock.clone(),
        ));
        let reading = Arc::new(DocumentReading::new(self.store.clone(), p.reader.clone()));
        let suggest_tags = Arc::new(SuggestTags::new(reading.clone(), p.assistant.clone(), p.tasks.clone()));
        AiServices {
            classify_all_new: Arc::new(ClassifyAllNew::new(self.store.clone(), p.tasks.clone(), suggest.clone())),
            suggest_classification: suggest,
            approve_suggestion: Arc::new(ApproveSuggestion::new(
                self.store.clone(),
                p.tasks.clone(),
                self.filing.clone(),
                self.activity.clone(),
                p.clock.clone(),
                p.ids.clone(),
            )),
            reject_suggestion: Arc::new(RejectSuggestion::new(p.tasks.clone())),
            list_ai_models: Arc::new(ListAiModels::new(p.ai_server.clone())),
            summarize_document: Arc::new(SummarizeDocument::new(reading.clone(), p.assistant.clone())),
            tag_all_untagged: Arc::new(TagAllUntagged::new(self.store.clone(), p.tasks.clone(), suggest_tags.clone())),
            suggest_tags,
            ask_about_document: Arc::new(AskAboutDocument::new(self.store.clone(), reading, p.assistant.clone())),
        }
    }

    fn organize_services(&self) -> OrganizeServices {
        let p = &self.ports;
        let (store, activity) = (&self.store, &self.activity);
        let apply = Arc::new(ApplyRestructurePlan::new(
            store.clone(),
            p.storage.clone(),
            self.replicator.clone(),
            p.plans.clone(),
            activity.clone(),
            p.clock.clone(),
        ));
        OrganizeServices {
            configure_inbox: Arc::new(ConfigureInbox::new(
                store.clone(),
                p.storage.clone(),
                self.replicator.clone(),
                activity.clone(),
            )),
            collect_inbox: Arc::new(CollectInbox::new(store.clone(), self.adopt_new_file.clone())),
            file_inbox_document: Arc::new(FileInboxDocument::new(store.clone(), self.filing.clone(), activity.clone())),
            folder_watcher: p.watcher.clone(),
            plan_restructure: Arc::new(PlanRestructure::new(store.clone(), self.matcher.clone())),
            reverse_restructure: Arc::new(ReverseRestructure::new(store.clone(), p.plans.clone(), apply.clone())),
            apply_restructure_plan: apply,
            list_restructure_history: Arc::new(ListRestructureHistory::new(store.clone(), p.plans.clone())),
            remove_empty_folders: Arc::new(RemoveEmptyFolders::new(
                store.clone(),
                p.storage.clone(),
                self.replicator.clone(),
                activity.clone(),
            )),
            accept_rule_proposal: Arc::new(AcceptRuleProposal::new(self.change_settings.clone(), p.ids.clone())),
            export_documents: Arc::new(ExportDocuments::new(
                store.clone(),
                self.matcher.clone(),
                p.exports.clone(),
                activity.clone(),
                p.clock.clone(),
            )),
            export_writer: p.exports.clone(),
        }
    }

    fn pdf_services(&self) -> PdfServices {
        let p = &self.ports;
        let (store, activity) = (&self.store, &self.activity);
        let versions = Arc::new(PdfVersions::new(
            store.clone(),
            p.storage.clone(),
            self.replicator.clone(),
            p.reader.clone(),
            p.texts.clone(),
            p.ids.clone(),
        ));
        let availability = Arc::new(RebuildAvailability::new(store.clone(), p.pdf_editor.clone()));
        let make_searchable = Arc::new(MakeSearchable::new(
            store.clone(),
            p.pdf_editor.clone(),
            p.ocr.clone(),
            p.texts.clone(),
            versions.clone(),
            activity.clone(),
        ));
        PdfServices {
            edit_pages: Arc::new(EditPages::new(
                store.clone(),
                p.pdf_editor.clone(),
                versions.clone(),
                activity.clone(),
            )),
            split_document: Arc::new(SplitDocument::new(
                store.clone(),
                p.pdf_editor.clone(),
                self.replicator.clone(),
                self.names.clone(),
                self.trash_document.clone(),
                activity.clone(),
                p.clock.clone(),
                p.ids.clone(),
            )),
            merge_documents: Arc::new(MergeDocuments::new(
                store.clone(),
                p.pdf_editor.clone(),
                self.replicator.clone(),
                self.names.clone(),
                self.trash_document.clone(),
                activity.clone(),
                p.clock.clone(),
                p.ids.clone(),
            )),
            make_all_searchable: Arc::new(MakeAllSearchable::new(store.clone(), make_searchable.clone())),
            make_searchable,
            scans_without_text: Arc::new(ScansWithoutText::new(store.clone(), p.pdf_editor.clone())),
            embed_pdf_metadata: Arc::new(EmbedPdfMetadata::new(store.clone(), p.pdf_editor.clone(), versions.clone())),
            editor: p.pdf_editor.clone(),
            ocr: p.ocr.clone(),
            rebuild_pdf: Arc::new(RebuildPdf::new(
                store.clone(),
                p.pdf_editor.clone(),
                p.rebuilder.clone(),
                p.layout_reader.clone(),
                availability.clone(),
            )),
            rebuild_availability: availability,
            keep_rebuild: Arc::new(KeepRebuild::new(
                store.clone(),
                p.rebuilder.clone(),
                versions,
                self.replicator.clone(),
                self.names.clone(),
                activity.clone(),
                p.clock.clone(),
                p.ids.clone(),
            )),
            rebuilder: p.rebuilder.clone(),
        }
    }

    fn ask_services(&self) -> AskServices {
        let p = &self.ports;
        let texts = Arc::new(DocumentTexts::new(p.reader.clone(), p.texts.clone()));
        let vectors = Arc::new(VectorSearch::new(p.index.clone()));
        AskServices {
            index_library: Arc::new(IndexLibrary::new(
                self.store.clone(),
                texts.clone(),
                p.embeddings.clone(),
                p.index.clone(),
                self.replicator.clone(),
            )),
            ask_library: Arc::new(AskLibrary::new(
                self.store.clone(),
                self.matcher.clone(),
                texts,
                p.embeddings.clone(),
                vectors.clone(),
                p.library_assistant.clone(),
            )),
            related_documents: Arc::new(RelatedDocuments::new(self.store.clone(), vectors)),
        }
    }
}
