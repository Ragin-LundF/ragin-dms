package dms.ai.support

import dms.application.port.ai.model.ClassificationRequestModel
import dms.application.port.ai.model.DocumentContent
import dms.domain.document.model.DocumentIdModel
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.library.model.RelativePathModel
import dms.domain.outline.model.FolderOutlineModel
import dms.domain.outline.model.OutlineFolderModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.LocalDate

/** Classification requests with synthetic names only. */
object Requests {

    val invoices: CategoryModel = CategoryModel(id = CategoryIdModel(value = "invoices"), name = "Invoices")
    val energy: CategoryModel = CategoryModel(
        id = CategoryIdModel(value = "energy"),
        name = "Energy",
        parentId = invoices.id,
    )

    /** An outline with `Home` and `Home/Power`, without the inbox. */
    val outline: FolderOutlineModel = FolderOutlineModel(
        folders = listOf(
            OutlineFolderModel(
                path = RelativePathModel.of(raw = "Home"),
                documentCount = 3,
                topCategory = energy.id,
                sampleTitles = listOf("Sample bill"),
            ),
            OutlineFolderModel(
                path = RelativePathModel.of(raw = "Home/Power"),
                documentCount = 2,
                topCategory = energy.id,
                sampleTitles = emptyList(),
                hiddenSubfolders = 4,
            ),
        ),
        excluded = listOf(RelativePathModel.of(raw = "Inbox")),
    )

    fun request(
        content: DocumentContent = DocumentContent.Text(text = "Sample text"),
        currentFileName: String = "scan-001.pdf",
        outline: FolderOutlineModel? = null,
        fieldDefinitions: List<FieldDefinitionModel> = emptyList(),
    ): ClassificationRequestModel {
        return ClassificationRequestModel(
            documentId = DocumentIdModel(value = "doc-1"),
            currentFileName = currentFileName,
            content = content,
            categories = listOf(invoices, energy),
            tags = setOf(TagNameModel.of(raw = "Tax")),
            folders = listOf(RelativePathModel.of(raw = "letters")),
            fileNameExamples = listOf(
                "2024-01-02 Sample - One.pdf",
                "2024-02-03 Sample - Two.pdf",
                "2024-03-04 Sample - Three.pdf",
            ),
            today = LocalDate.parse("2026-01-15"),
            outline = outline,
            fieldDefinitions = fieldDefinitions,
            defaultCurrency = CurrencyCodeModel(value = "EUR"),
        )
    }
}
