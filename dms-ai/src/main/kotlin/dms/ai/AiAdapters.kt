package dms.ai

import dms.ai.client.OpenAiCompatibleClient
import dms.ai.read.FileContentReader
import dms.application.port.state.model.AiSettings
import dms.domain.library.LibraryFolder
import dms.domain.library.RelativePath
import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import java.nio.file.Path

/**
 * The AI adapters: the client for an OpenAI-compatible server and the document reader. [settings]
 * is read for every call. [locate] finds a library file on disk. Tests pass their own [engine]; the
 * default talks HTTP through CIO.
 */
class AiAdapters(
    settings: suspend () -> AiSettings,
    locate: (LibraryFolder, RelativePath) -> Path?,
    engine: HttpClientEngine? = null,
) {
    private val http: HttpClient = HttpClient(engine = engine ?: CIO.create()) {
        install(plugin = ContentNegotiation) {
            json(json = Json { ignoreUnknownKeys = true })
        }
        install(plugin = HttpTimeout) {
            connectTimeoutMillis = CONNECT_TIMEOUT_MILLIS
        }
    }

    val client: OpenAiCompatibleClient = OpenAiCompatibleClient(http = http, settings = settings)
    val reader: FileContentReader = FileContentReader(locate = locate, settings = settings)

    private companion object {
        const val CONNECT_TIMEOUT_MILLIS = 5_000L
    }
}
