/// Creates new unique ids for documents, categories, folders, libraries and journal entries.
pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> String;
}
