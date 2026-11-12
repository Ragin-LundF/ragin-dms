use crate::split::{SplitPart, SplitProblem};

/// The parts a stack of `page_count` pages splits into. When the proposed parts did not cover every page in order,
/// the proposal is the whole stack as one part and [`Self::problem`] says why.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitProposal {
    pub parts: Vec<SplitPart>,
    pub problem: Option<SplitProblem>,
}

impl SplitProposal {
    /// `parts` as they are when they cover every page of the stack in order, without gaps or overlaps; otherwise
    /// one part of all pages with the problem found.
    #[must_use]
    pub fn checked(parts: Vec<SplitPart>, page_count: usize) -> Self {
        match Self::problem_of(&parts, page_count) {
            None => Self { parts, problem: None },
            Some(problem) => {
                Self { parts: vec![SplitPart::pages(0, page_count.saturating_sub(1))], problem: Some(problem) }
            }
        }
    }

    /// Why `parts` do not cover pages `0..page_count` one after the other, or `None` if they do.
    #[must_use]
    pub fn problem_of(parts: &[SplitPart], page_count: usize) -> Option<SplitProblem> {
        if parts.is_empty() || page_count == 0 {
            return Some(SplitProblem::NoParts);
        }
        let mut next = 0;
        for part in parts {
            if part.first_page < next || part.last_page < part.first_page {
                return Some(SplitProblem::Overlap);
            }
            if part.first_page > next {
                return Some(SplitProblem::Gap);
            }
            if part.last_page >= page_count {
                return Some(SplitProblem::BeyondLastPage);
            }
            next = part.last_page + 1;
        }
        (next < page_count).then_some(SplitProblem::Gap)
    }
}
