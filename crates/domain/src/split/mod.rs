//! Splitting a scanned stack: where each document of the stack starts, as the AI proposes it, and the check that
//! the parts cover every page in order.

pub mod model;
mod split_problem;

pub use model::{SplitPart, SplitProposal};
pub use split_problem::SplitProblem;
