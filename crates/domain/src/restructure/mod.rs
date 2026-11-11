//! The restructure assistant: path templates, plans and folders that would profit from a restructure.

mod file_naming;
mod folder_shape;
pub mod model;
mod restructure_planner;
mod skip_reason;
pub mod template;

pub use file_naming::FileNaming;
pub use folder_shape::FolderShape;
pub use model::{FlatFolder, PlannedMove, RestructurePlan, SkippedDocument};
pub use restructure_planner::RestructurePlanner;
pub use skip_reason::SkipReason;
pub use template::{PathTemplate, PlaceholderName, TemplateParse, TemplatePart, TemplateProblem, TemplateRendering};
