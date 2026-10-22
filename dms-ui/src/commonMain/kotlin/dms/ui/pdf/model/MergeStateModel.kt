package dms.ui.pdf.model

import dms.ui.document.details.model.LinkCandidateModel

/**
 * "Merge with…": the PDFs to merge in their order ([chosen], the first is the document the user
 * started from), the search for more, the new file's name and whether the sources go to the trash.
 */
data class MergeStateModel(
    val chosen: List<LinkCandidateModel>,
    val query: String = "",
    val matches: List<LinkCandidateModel> = emptyList(),
    val name: String = "",
    val trashSources: Boolean = false,
)
