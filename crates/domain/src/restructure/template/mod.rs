//! Path templates such as `{category}/{year|Undated}`.

pub mod model;
mod placeholder_name;
mod template_parse;
mod template_part;
mod template_problem;
mod template_rendering;

pub use model::PathTemplate;
pub use placeholder_name::PlaceholderName;
pub use template_parse::TemplateParse;
pub use template_part::TemplatePart;
pub use template_problem::TemplateProblem;
pub use template_rendering::TemplateRendering;
