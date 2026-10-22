package dms.domain.query.model

/** A named [DocumentQueryModel], shown as a smart folder and usable as the scope of restructure and export. */
data class SavedSearchModel(
    val id: SavedSearchIdModel,
    val name: String,
    val query: DocumentQueryModel,
) {
    init {
        require(value = name.isNotBlank()) { "Saved search name must not be blank" }
    }
}
