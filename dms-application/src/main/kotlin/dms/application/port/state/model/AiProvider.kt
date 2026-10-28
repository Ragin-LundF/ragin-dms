package dms.application.port.state.model

/** Which AI answers: a local OpenAI-compatible server such as LM Studio, or Anthropic's API. */
enum class AiProvider {
    LOCAL,
    ANTHROPIC,
}
