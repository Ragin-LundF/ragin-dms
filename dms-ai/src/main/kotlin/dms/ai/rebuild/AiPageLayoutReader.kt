package dms.ai.rebuild

import dms.ai.client.ChatRequestBuilder
import dms.ai.client.ChatTransport
import dms.ai.client.model.ChatMessageModel
import dms.ai.client.model.ChatReply
import dms.ai.prompt.PromptTemplates
import dms.ai.prompt.RebuildLayoutSchema
import dms.ai.read.VisionImages
import dms.ai.rebuild.dto.LayoutAnswerDto
import dms.application.port.ai.PageLayoutReader
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.PageImageModel
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonPrimitive
import java.io.ByteArrayInputStream
import javax.imageio.ImageIO

/**
 * Reads page layouts with the configured vision model. The page goes out as PNG on the patch grid of
 * Qwen-VL models, the answer is constrained by [RebuildLayoutSchema]. A line is read again without a
 * schema: some models write NUL for every umlaut when the answer is forced into a JSON grammar. The
 * document text is never logged.
 */
class AiPageLayoutReader(private val transport: ChatTransport) : PageLayoutReader {

    override suspend fun readLayout(page: PageImageModel): LayoutOutcome {
        val sent = withContext(context = Dispatchers.IO) {
            ImageIO.read(ByteArrayInputStream(page.bytes))?.let { VisionImages.layoutPng(image = it) }
        } ?: return LayoutOutcome.Failure(reason = ClassificationFailure.INVALID_RESPONSE, detail = NOT_AN_IMAGE)
        val image = VisionImages.pngOf(image = sent)
        val request = PromptTemplates.fill(
            name = "rebuild-layout-request.md",
            values = mapOf("width" to sent.width.toString(), "height" to sent.height.toString()),
        )
        val system = PromptTemplates.fill(name = "rebuild-layout.md", values = emptyMap())
        val messages = listOf(
            ChatMessageModel(role = "system", content = JsonPrimitive(value = system)),
            ChatMessageModel(
                role = "user",
                content = ChatRequestBuilder.content(text = request, images = listOf(image)),
            ),
        )
        val reply = transport.complete(
            hasImages = true,
            structured = true,
            minTimeoutSeconds = LAYOUT_TIMEOUT_SECONDS,
        ) { model ->
            ChatRequestBuilder.structured(
                model = model,
                temperature = TEMPERATURE,
                messages = messages,
                schemaName = "page_layout",
                schema = RebuildLayoutSchema.schema,
            )
        }
        return when (reply) {
            is ChatReply.Failed -> LayoutOutcome.Failure(reason = reply.reason, detail = reply.detail)
            is ChatReply.Answered -> {
                val answer = parse(content = reply.content) ?: return LayoutOutcome.Failure(
                    reason = ClassificationFailure.INVALID_RESPONSE,
                    detail = "The answer is no page layout",
                )
                val layout = LayoutMapper.toLayout(answer = answer, sentWidth = sent.width, sentHeight = sent.height)
                LayoutOutcome.Read(layout = layout)
            }
        }
    }

    override suspend fun readLine(line: PageImageModel): LineOutcome {
        val text = PromptTemplates.fill(name = "rebuild-line.md", values = emptyMap())
        val content = ChatRequestBuilder.content(text = text, images = listOf(line))
        val messages = listOf(ChatMessageModel(role = "user", content = content))
        val reply = transport.complete(hasImages = true) { model ->
            ChatRequestBuilder.chat(model = model, temperature = TEMPERATURE, messages = messages)
        }
        return when (reply) {
            is ChatReply.Failed -> LineOutcome.Failure(reason = reply.reason, detail = reply.detail)
            is ChatReply.Answered -> lineOf(content = reply.content)
                ?.let { LineOutcome.Read(text = it) }
                ?: LineOutcome.Failure(reason = ClassificationFailure.INVALID_RESPONSE, detail = NO_TEXT)
        }
    }

    /** The outermost JSON object; a code fence or a sentence around it is dropped. `null` if it is no layout. */
    private fun parse(content: String): LayoutAnswerDto? {
        val start = content.indexOf(char = '{')
        val end = content.lastIndexOf(char = '}')
        if (start < 0 || end <= start) return null
        val json = content.substring(startIndex = start, endIndex = end + 1)
        return runCatching { JSON.decodeFromString<LayoutAnswerDto>(string = json) }.getOrNull()
    }

    /** The last line of text without a code fence: models sometimes explain first. */
    private fun lineOf(content: String): String? {
        return content.lines()
            .map { it.trim() }
            .filterNot { it.startsWith(prefix = FENCE) }
            .lastOrNull { it.isNotEmpty() }
    }

    private companion object {
        /** The pages took 1–4 minutes each with a 27B model, so the configured timeout is often too short. */
        const val LAYOUT_TIMEOUT_SECONDS = 600
        const val TEMPERATURE = 0.0
        const val FENCE = "```"
        const val NOT_AN_IMAGE = "Not an image"
        const val NO_TEXT = "The AI answered nothing"
        val JSON = Json {
            ignoreUnknownKeys = true
            isLenient = true
            coerceInputValues = true
        }
    }
}
