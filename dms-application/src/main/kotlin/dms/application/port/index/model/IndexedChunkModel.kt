package dms.application.port.index.model

/** One passage of a document's text, from character [start] to [end], with its embedding [vector]. */
class IndexedChunkModel(
    val start: Int,
    val end: Int,
    val vector: FloatArray,
)
