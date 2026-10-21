package dms.application.fake.ai

import dms.application.port.ai.DocumentClassifier
import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.ClassificationResult
import dms.domain.classification.ClassificationSuggestion

/** Answers with [result], or with an empty suggestion for the requested document. Records every request. */
class FakeClassifier : DocumentClassifier {

    var result: ClassificationResult? = null
    val requests: MutableList<ClassificationRequest> = mutableListOf()

    override suspend fun suggest(request: ClassificationRequest): ClassificationResult {
        requests += request
        return result ?: ClassificationResult.Suggested(
            suggestion = ClassificationSuggestion(documentId = request.documentId, model = MODEL),
        )
    }

    companion object {
        const val MODEL: String = "test-model"
    }
}
