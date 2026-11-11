use std::collections::HashMap;

use dms_application::port::ai::{DocumentPreviewer, PageImage};
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::support::Shared;

/// Draws the `pictures` by file path; every other file has none. Records each request in `requests`.
#[derive(Default)]
pub struct FakePreviewer {
    pub pictures: Shared<HashMap<RelativePath, PageImage>>,
    pub requests: Shared<Vec<PreviewRequest>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRequest {
    pub folder: LibraryFolder,
    pub path: RelativePath,
    pub max_side: u32,
}

impl DocumentPreviewer for FakePreviewer {
    fn thumbnail(&self, folder: &LibraryFolder, path: &RelativePath, max_side: u32) -> Option<PageImage> {
        self.requests.update(|requests| {
            requests.push(PreviewRequest { folder: folder.clone(), path: path.clone(), max_side });
        });
        self.pictures.lock().get(path).cloned()
    }
}
