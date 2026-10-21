package dms.ui.assistant

import dms.application.assistant.model.AnswerOutcome
import dms.application.port.assistant.model.ChatRole
import dms.application.port.assistant.model.ChatTurn
import dms.domain.document.Document
import dms.ui.assistant.model.ChatState
import dms.ui.review.AiMessages
import dms.ui.service.AppServices
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * A conversation with the AI about one document; the user writes the questions. It only reads, so
 * it runs beside whatever the library does. A question the AI could not answer is taken back, so
 * the conversation always alternates between the user and the AI.
 */
class ChatController(
    private val services: AppServices,
    private val scope: CoroutineScope,
) {

    private val mutableState = MutableStateFlow<ChatState?>(value = null)
    private var job: Job? = null

    val state: StateFlow<ChatState?> = mutableState.asStateFlow()

    fun open(document: Document) {
        if (mutableState.value?.document?.id != document.id) {
            job?.cancel()
            mutableState.value = ChatState(document = document)
        }
    }

    fun type(input: String) {
        mutableState.update { it?.copy(input = input) }
    }

    fun send(question: String = mutableState.value?.input.orEmpty()) {
        val chat = mutableState.value ?: return
        val text = question.trim()
        if (text.isEmpty() || chat.waiting) {
            return
        }
        val asked = chat.turns + ChatTurn(role = ChatRole.USER, text = text)
        mutableState.value = chat.copy(turns = asked, input = "", waiting = true, problem = null)
        job = scope.launch {
            val outcome = services.ai.askAboutDocument(documentId = chat.document.id, conversation = asked)
            mutableState.update { current ->
                when {
                    current == null -> null
                    outcome is AnswerOutcome.Answered -> current.copy(
                        turns = asked + ChatTurn(role = ChatRole.ASSISTANT, text = outcome.text),
                        waiting = false,
                    )

                    else -> current.copy(
                        turns = chat.turns,
                        input = text,
                        waiting = false,
                        problem = AiMessages.problemOf(outcome = outcome, name = chat.document.path.fileName),
                    )
                }
            }
        }
    }

    /** Stops the AI; the unanswered question goes back into the input. */
    fun stop() {
        val chat = mutableState.value ?: return
        if (!chat.waiting) {
            return
        }
        job?.cancel()
        val question = chat.turns.lastOrNull()?.takeIf { it.role == ChatRole.USER }
        mutableState.value = chat.copy(
            turns = if (question != null) chat.turns.dropLast(n = 1) else chat.turns,
            input = question?.text ?: chat.input,
            waiting = false,
        )
    }

    fun clear() {
        stop()
        mutableState.update { it?.copy(turns = emptyList(), problem = null) }
    }

    fun close() {
        job?.cancel()
        mutableState.value = null
    }
}
