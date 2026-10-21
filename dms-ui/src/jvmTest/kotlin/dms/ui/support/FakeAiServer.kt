package dms.ui.support

import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.serialization.json.Json
import java.net.ConnectException

/**
 * Answers in place of LM Studio: [models] for the model list, and [suggestion] (the model's JSON)
 * for every chat request. While [reachable] is false, every request fails like a stopped server.
 */
class FakeAiServer {

    var models: List<String> = listOf("sample-vision-model")
    var suggestion: String = "{}"
    var reachable: Boolean = true
    var chatRequests: Int = 0
        private set

    val engine: HttpClientEngine = MockEngine { request ->
        if (!reachable) {
            throw ConnectException("Connection refused")
        }
        val body = if (request.url.encodedPath.endsWith(suffix = "/models")) {
            val data = models.joinToString(separator = ",") { """{"id":${Json.encodeToString(value = it)}}""" }
            """{"data":[$data]}"""
        } else {
            chatRequests++
            """{"choices":[{"message":{"role":"assistant","content":${Json.encodeToString(value = suggestion)}}}]}"""
        }
        respond(
            content = body,
            status = HttpStatusCode.OK,
            headers = headersOf(HttpHeaders.ContentType, "application/json"),
        )
    }
}
