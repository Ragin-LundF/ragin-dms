package dms.application.fake.ai

import dms.application.port.ai.DocumentClassifier
import dms.application.port.ai.model.ClassificationRequestModel
import dms.application.port.ai.model.ClassificationResult
import dms.domain.classification.model.ClassificationSuggestionModel

/** Answers with [result], or with an empty suggestion for the requested document. Records every request. */
class FakeClassifier : DocumentClassifier {

    var result: ClassificationResult? = null
    val requests: MutableList<ClassificationRequestModel> = mutableListOf()

    override suspend fun suggest(request: ClassificationRequestModel): ClassificationResult {
        requests += request
        return result ?: ClassificationResult.Suggested(
            suggestion = ClassificationSuggestionModel(documentId = request.documentId, model = MODEL),
        )
    }

    companion object {
        const val MODEL: String = "test-model"
    }
}
