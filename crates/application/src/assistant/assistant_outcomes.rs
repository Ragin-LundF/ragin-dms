use crate::assistant::{AnswerOutcome, ReadingResult};
use crate::port::ai::ClassificationFailure;
use crate::port::assistant::AssistantAnswer;

/// Maps reading problems and AI answers to the outcomes the UI shows.
pub struct AssistantOutcomes;

impl AssistantOutcomes {
    /// # Panics
    /// For [`ReadingResult::Read`], which is no problem (Kotlin: `error(..)`).
    #[must_use]
    pub fn of_problem(problem: ReadingResult) -> AnswerOutcome {
        match problem {
            ReadingResult::Read { .. } => panic!("A document that was read is no problem"),
            ReadingResult::LibraryNotOpen => AnswerOutcome::LibraryNotOpen,
            ReadingResult::NotFound => AnswerOutcome::NotFound,
            ReadingResult::NotReadable { detail } => AnswerOutcome::NotReadable { detail },
        }
    }

    #[must_use]
    pub fn of_answer(answer: AssistantAnswer) -> AnswerOutcome {
        match answer {
            AssistantAnswer::Text { text, model } => AnswerOutcome::Answered { text, model },
            AssistantAnswer::Failed { reason, detail } => AnswerOutcome::Failed { reason, detail },
            AssistantAnswer::Tags { .. } => AnswerOutcome::Failed {
                reason: ClassificationFailure::InvalidResponse,
                detail: Some("Tags where text was expected".to_owned()),
            },
        }
    }
}
