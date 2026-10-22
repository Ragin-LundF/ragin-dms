package dms.domain.outline.model

import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel

/**
 * One folder of a [FolderOutlineModel]: how many documents it holds, directly or below it, its most common
 * category, a few sample titles, and how many subfolders were left out to keep the outline short.
 */
data class OutlineFolderModel(
    val path: RelativePathModel,
    val documentCount: Int,
    val topCategory: CategoryIdModel?,
    val sampleTitles: List<String>,
    val hiddenSubfolders: Int = 0,
)
