/// Which AI answers: a local OpenAI-compatible server such as LM Studio, or Anthropic's API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AiProvider {
    #[default]
    Local,
    Anthropic,
}
