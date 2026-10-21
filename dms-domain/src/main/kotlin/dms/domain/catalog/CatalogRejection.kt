package dms.domain.catalog

/** Why a catalog change was refused. The catalog stays unchanged. */
enum class CatalogRejection {
    DOCUMENT_ALREADY_EXISTS,
    DOCUMENT_NOT_FOUND,
    PATH_ALREADY_USED,
    INVALID_FOLDER_MOVE,
    CATEGORY_ALREADY_EXISTS,
    CATEGORY_NOT_FOUND,
    CATEGORY_NAME_TAKEN,
    CATEGORY_IN_USE,
    CATEGORY_HAS_CHILDREN,
    CATEGORY_CYCLE,
    TAG_NOT_FOUND,
}
