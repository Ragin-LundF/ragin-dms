package dms.ai.client

import kotlin.test.Test
import kotlin.test.assertEquals

class AnswerTextTest {

    @Test
    fun `thoughts and bold markers are removed`() {
        val answer = "<think>Let me check the date.</think>\nBis **31.03.2024** zahlen."

        assertEquals(expected = "Bis 31.03.2024 zahlen.", actual = AnswerText.plain(content = answer))
    }

    @Test
    fun `an answer after a lone closing think tag is kept`() {
        assertEquals(expected = "Answer", actual = AnswerText.clean(content = "thoughts</think> Answer"))
    }
}
