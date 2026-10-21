package dms.application.port.ai

import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.ClassificationResult

/** Suggests title, category, tags, date, correspondent and file name. Suggestions only, never applied. */
interface DocumentClassifier {

    suspend fun suggest(request: ClassificationRequest): ClassificationResult
}
