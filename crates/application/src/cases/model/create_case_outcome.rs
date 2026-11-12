use dms_domain::cases::CaseId;

use crate::common::OperationOutcome;

/// The id the new case got, valid when `outcome` is done.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateCaseOutcome {
    pub case_id: CaseId,
    pub outcome: OperationOutcome,
}
