use dms_domain::restructure::RestructurePlan;
use dms_domain::restructure::template::TemplateProblem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanOutcome {
    Planned(RestructurePlan),
    InvalidTemplate(TemplateProblem),
    LibraryNotOpen,
}
