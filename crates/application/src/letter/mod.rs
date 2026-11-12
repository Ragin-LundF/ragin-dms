//! Letters: a draft by the AI or from a fixed template, edited by the user and saved as a catalogued PDF linked to
//! its source documents.

mod draft_letter;
mod language_detection;
mod letter_layout;
mod letter_template;
pub mod model;
mod save_letter;

pub use draft_letter::DraftLetter;
pub use language_detection::detect_language;
pub use letter_layout::letter_document;
pub use letter_template::template_draft;
pub use model::{DraftLetterInput, DraftLetterOutcome, DraftSource, SaveLetterInput, SaveLetterOutcome};
pub use save_letter::SaveLetter;
