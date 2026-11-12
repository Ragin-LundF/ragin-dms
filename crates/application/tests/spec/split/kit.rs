use std::ops::Deref;
use std::sync::Arc;

use dms_application::pdf::SplitDocument;
use dms_application::split::{ApplySplit, ProposeSplit};

use crate::support::TestLibrary;
use crate::support::fake::assistant::FakeSplitProposer;
use crate::support::fake::pdf::FakePdfEditor;

/// An open `TestLibrary` with the PDF editor and split proposer fakes.
pub struct SplitKit {
    pub library: TestLibrary,
    pub pdf: Arc<FakePdfEditor>,
    pub proposer: Arc<FakeSplitProposer>,
}

impl Deref for SplitKit {
    type Target = TestLibrary;

    fn deref(&self) -> &TestLibrary {
        &self.library
    }
}

impl SplitKit {
    pub fn new() -> Self {
        let library = TestLibrary::new();
        library.open();
        Self {
            pdf: Arc::new(FakePdfEditor::new(library.folders.clone())),
            proposer: Arc::new(FakeSplitProposer::default()),
            library,
        }
    }

    pub fn propose_split(&self) -> ProposeSplit {
        ProposeSplit::new(self.store.clone(), self.pdf.clone(), self.folders.clone(), self.proposer.clone())
    }

    pub fn apply_split(&self) -> ApplySplit {
        let split = SplitDocument::new(
            self.store.clone(),
            self.pdf.clone(),
            self.replicator.clone(),
            self.names.clone(),
            Arc::new(self.trash_document()),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        );
        ApplySplit::new(self.store.clone(), self.pdf.clone(), Arc::new(split), self.tasks.clone())
    }
}
