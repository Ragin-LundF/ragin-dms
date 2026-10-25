package dms.application.classification

import dms.application.fake.ai.FakeAiServer
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettingsModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class ListAiModelsTest {

    private val server = FakeAiServer()
    private val listAiModels = ListAiModels(server = server)

    @Test
    fun `asks the server with the given settings and returns its models`() {
        runTest {
            val settings = AiSettingsModel(baseUrl = "http://localhost:9999/v1", model = "sample-model")
            server.answer = ModelList.Available(models = listOf("sample-model", "sample-vision"))

            val models = listAiModels(settings = settings)

            assertEquals(
                expected = ModelList.Available(models = listOf("sample-model", "sample-vision")),
                actual = models,
            )
            assertEquals(expected = listOf(settings), actual = server.requests)
        }
    }

    @Test
    fun `an unreachable server is reported as unavailable with its reason`() {
        runTest {
            val unavailable = ModelList.Unavailable(
                reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                detail = "connection refused",
            )
            server.answer = unavailable

            val models = listAiModels(settings = AiSettingsModel())

            assertEquals(expected = unavailable, actual = models)
        }
    }

    @Test
    fun `a server without models answers with an empty list`() {
        runTest {
            server.answer = ModelList.Available(models = emptyList())

            val models = listAiModels(settings = AiSettingsModel())

            assertEquals(expected = ModelList.Available(models = emptyList()), actual = models)
        }
    }
}
