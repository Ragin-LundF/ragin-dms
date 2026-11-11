use dms_domain::library::RelativePath;

/// Where a scan of the library stands: folder `folder_number` of `folder_count` is being read, `done` of its
/// `total` files are finished. `total` is `None` while the folder is still being listed. `current` is the last
/// finished file, and `hashed` tells whether its content had to be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanProgress {
    pub folder_name: String,
    pub folder_number: usize,
    pub folder_count: usize,
    pub done: usize,
    pub total: Option<usize>,
    pub current: Option<RelativePath>,
    pub hashed: bool,
}
