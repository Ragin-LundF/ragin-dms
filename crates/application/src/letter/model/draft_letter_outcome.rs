use dms_domain::library::RelativePath;

use crate::letter::DraftSource;
use crate::port::assistant::{LetterDraft, LetterLanguage, LetterParty};

/// A draft to show in the letter dialog, with the recipient as far as known, the letter's language, the folder it
/// is saved to unless the user picks another (that of the newest source, else the top level), and who wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftLetterOutcome {
    Drafted {
        draft: LetterDraft,
        recipient: LetterParty,
        language: LetterLanguage,
        folder: RelativePath,
        source: DraftSource,
    },
    LibraryNotOpen,
    /// A source document is not in the catalog (any more).
    NotFound,
}
