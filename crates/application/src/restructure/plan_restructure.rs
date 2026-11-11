use std::sync::Arc;

use dms_domain::document::Document;
use dms_domain::restructure::template::{PathTemplate, TemplateParse};
use dms_domain::restructure::{FileNaming, RestructurePlanner};

use crate::library::CatalogStore;
use crate::restructure::{PlanOutcome, RestructureScope};
use crate::search::ContentMatcher;

/// Plans a restructure of a scope by a path template, for the user to review. Changes nothing.
pub struct PlanRestructure {
    store: Arc<CatalogStore>,
    search: Arc<ContentMatcher>,
}

impl PlanRestructure {
    pub fn new(store: Arc<CatalogStore>, search: Arc<ContentMatcher>) -> Self {
        Self { store, search }
    }

    pub fn run(&self, scope: &RestructureScope, template: &str, naming: FileNaming) -> PlanOutcome {
        let parsed = match PathTemplate::parse(template) {
            TemplateParse::Invalid(problem) => return PlanOutcome::InvalidTemplate(problem),
            TemplateParse::Parsed(template) => template,
        };
        let Some(library) = self.store.current() else {
            return PlanOutcome::LibraryNotOpen;
        };
        let catalog = &library.catalog;
        let matched: Vec<Document>;
        let documents: Vec<&Document> = match scope {
            RestructureScope::Folder(path) => {
                catalog.documents().values().filter(|document| document.path.is_inside(path)).collect()
            }
            RestructureScope::Query(query) => {
                matched = self.search.matching(&library, query);
                matched.iter().collect()
            }
        };
        let plan = RestructurePlanner::plan(catalog, &documents, &parsed, naming, &catalog.settings().layout.inbox);
        PlanOutcome::Planned(plan)
    }
}
