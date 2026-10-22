package dms.ai

import dms.ai.assistant.OpenAiAssistant
import dms.ai.assistant.OpenAiLibraryAssistant
import dms.ai.client.ChatTransport
import dms.ai.client.OpenAiCompatibleClient
import dms.ai.client.OpenAiEmbeddings
import dms.ai.read.FileContentReader
import dms.ai.read.FilePreviewer
import dms.application.port.state.model.AiSettingsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import java.nio.file.Path

/**
 * The AI adapters: the classifier and the assistant on an OpenAI-compatible server, the
 * document reader and the thumbnails. [settings] is read for every call. [locate] finds a library
 * file on disk. Tests pass their own [engine]; the default talks HTTP through CIO.
 */
class AiAdapters(
    settings: suspend () -> AiSettingsModel,
    locate: (LibraryFolderModel, RelativePathModel) -> Path?,
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

    private val transport: ChatTransport = ChatTransport(http = http, settings = settings)

    val client: OpenAiCompatibleClient = OpenAiCompatibleClient(transport = transport)
    val assistant: OpenAiAssistant = OpenAiAssistant(transport = transport)
    val libraryAssistant: OpenAiLibraryAssistant = OpenAiLibraryAssistant(transport = transport)
    val embeddings: OpenAiEmbeddings = OpenAiEmbeddings(http = http, transport = transport, settings = settings)
    val reader: FileContentReader = FileContentReader(locate = locate, settings = settings)
    val previewer: FilePreviewer = FilePreviewer(locate = locate)

    private companion object {
        const val CONNECT_TIMEOUT_MILLIS = 5_000L
    }
}
