/// Why the classifier could not produce a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassificationFailure {
    ProviderUnavailable,
    ModelCannotReadImages,
    InvalidResponse,
    Timeout,
}
