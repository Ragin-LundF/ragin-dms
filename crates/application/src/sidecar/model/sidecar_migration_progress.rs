/// `done` of the `total` flat files of the folder `folder_name` are handled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarMigrationProgress {
    pub folder_name: String,
    pub done: usize,
    pub total: usize,
}
