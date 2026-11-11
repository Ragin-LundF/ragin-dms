/// Why a catalog change was refused. The catalog stays unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogRejection {
    DocumentAlreadyExists,
    DocumentNotFound,
    PathAlreadyUsed,
    InvalidFolderMove,
    CategoryAlreadyExists,
    CategoryNotFound,
    CategoryNameTaken,
    CategoryInUse,
    CategoryHasChildren,
    CategoryCycle,
    TagNotFound,
    FieldNotDefined,
    FieldTypeMismatch,
    LinkNotFound,
}
