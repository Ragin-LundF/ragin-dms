package dms.application.port.ai.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.library.model.RelativePathModel
import dms.domain.outline.model.FolderOutlineModel
import dms.domain.rule.match.model.RuleResultModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.LocalDate

/**
 * Everything the classifier gets for one document: its content and the library's existing taxonomy,
 * folders and file name examples, so it prefers what the library already uses. With an [outline],
 * the classifier also suggests the folder to file the document in. It fills the library's custom
 * [fieldDefinitions] where the document has the values; a money value without a currency takes
 * [defaultCurrency]. [fixedValues] are what the user's matching rules decided; the classifier keeps
 * them. Retention is never part of a request: it is strictly the user's decision.
 */
data class ClassificationRequestModel(
    val documentId: DocumentIdModel,
    val currentFileName: String,
    val content: DocumentContent,
    val categories: List<CategoryModel>,
    val tags: Set<TagNameModel>,
    val folders: List<RelativePathModel>,
    val fileNameExamples: List<String>,
    val today: LocalDate,
    val outline: FolderOutlineModel? = null,
    val fieldDefinitions: List<FieldDefinitionModel> = emptyList(),
    val defaultCurrency: CurrencyCodeModel? = null,
    val fixedValues: RuleResultModel? = null,
)
