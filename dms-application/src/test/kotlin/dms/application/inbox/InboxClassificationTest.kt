package dms.application.inbox

import dms.application.TestLibrary
import dms.application.classification.model.SuggestOutcome
import dms.application.classification.model.SuggestionApprovalModel
import dms.application.common.OperationOutcome
import dms.application.fake.ai.FakeClassifier
import dms.application.port.ai.model.ClassificationResult
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedFolderModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.settings.model.InboxSettingsModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class InboxClassificationTest {

    private suspend fun libraryWithInboxDocument(aiTargetFolder: Boolean = true): Pair<TestLibrary, DocumentModel> {
        val library = TestLibrary()
        library.open()
        library.add(path = "Home/Power/old bill.pdf", content = "old")
        library.configureInbox(inbox = InboxSettingsModel(enabled = true, aiTargetFolder = aiTargetFolder))
        return library to library.add(path = "Inbox/scan.pdf", content = "new")
    }

    private fun suggestFolder(library: TestLibrary, document: DocumentModel, folder: String, isNew: Boolean) {
        library.classifier.result = ClassificationResult.Suggested(
            suggestion = ClassificationSuggestionModel(
                documentId = document.id,
                model = FakeClassifier.MODEL,
                targetFolder = SuggestedFolderModel(path = RelativePathModel.of(raw = folder), isNew = isNew),
                targetReason = "Other power bills are there",
            ),
        )
    }

    @Test
    fun `an inbox document gets the folder outline without the inbox`() {
        runTest {
            val (library, document) = libraryWithInboxDocument()

            library.suggestClassification(documentId = document.id)

            val outline = assertNotNull(actual = library.classifier.requests.single().outline)
            assertEquals(expected = listOf("Home", "Home/Power"), actual = outline.folders.map { it.path.value })
        }
    }

    @Test
    fun `without the target-folder setting there is no outline`() {
        runTest {
            val (library, document) = libraryWithInboxDocument(aiTargetFolder = false)

            library.suggestClassification(documentId = document.id)

            assertNull(actual = library.classifier.requests.single().outline)
        }
    }

    @Test
    fun `an existing and a new target folder are kept, one outside the outline is dropped`() {
        runTest {
            val (library, document) = libraryWithInboxDocument()

            suggestFolder(library = library, document = document, folder = "Home/Power", isNew = true)
            val existing = assertIs<SuggestOutcome.Suggested>(
                value = library.suggestClassification(documentId = document.id),
            )
            suggestFolder(library = library, document = document, folder = "Home/Water", isNew = true)
            val created = assertIs<SuggestOutcome.Suggested>(
                value = library.suggestClassification(documentId = document.id),
            )
            suggestFolder(library = library, document = document, folder = "Away/Water", isNew = true)
            val outside = assertIs<SuggestOutcome.Suggested>(
                value = library.suggestClassification(documentId = document.id),
            )

            assertEquals(
                expected = SuggestedFolderModel(path = RelativePathModel.of(raw = "Home/Power"), isNew = false),
                actual = existing.suggestion.targetFolder,
            )
            assertEquals(expected = true, actual = created.suggestion.targetFolder?.isNew)
            assertNull(actual = outside.suggestion.targetFolder)
            assertNull(actual = outside.suggestion.targetReason)
        }
    }

    @Test
    fun `approving files the document into the target folder with the new name in one move`() {
        runTest {
            val (library, document) = libraryWithInboxDocument()
            suggestFolder(library = library, document = document, folder = "Home/Water", isNew = true)
            library.suggestClassification(documentId = document.id)

            val outcome = library.approveSuggestion(
                documentId = document.id,
                approval = SuggestionApprovalModel(
                    fileName = "Water bill.pdf",
                    targetFolder = RelativePathModel.of(raw = "Home/Water"),
                ),
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(
                expected = "Home/Water/Water bill.pdf",
                actual = library.catalog.documents.getValue(document.id).path.value,
            )
            library.allFolders.forEach {
                assertEquals(
                    expected = "new",
                    actual = library.folders.content(folder = it, path = "Home/Water/Water bill.pdf"),
                )
            }
        }
    }
}
