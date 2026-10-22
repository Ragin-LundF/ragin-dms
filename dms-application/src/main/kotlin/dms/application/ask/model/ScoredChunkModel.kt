package dms.application.ask.model

import dms.domain.document.model.DocumentIdModel

/** A passage found for a question, with how well it fits: the higher [score], the better. */
data class ScoredChunkModel(
    val documentId: DocumentIdModel,
    val start: Int,
    val end: Int,
    val score: Double,
)
