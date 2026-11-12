//! The badges of the "Organize" screens in the navigation.

use dms_domain::task::OpenTask;

/// Overdue bills ("To pay") and follow-ups due ("Cases") among the dashboard's open tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrganizeBadges {
    pub to_pay: usize,
    pub cases: usize,
}

pub fn organize_badges(tasks: &[OpenTask]) -> OrganizeBadges {
    let count = |wanted: fn(&OpenTask) -> bool| tasks.iter().filter(|task| wanted(task)).count();
    OrganizeBadges {
        to_pay: count(|task| matches!(task, OpenTask::BillOverdue { .. })),
        cases: count(|task| matches!(task, OpenTask::FollowUpDue { .. })),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use dms_domain::cases::{CaseId, FollowUpId};
    use dms_domain::document::DocumentId;

    use super::*;

    #[test]
    fn badges_count_overdue_bills_and_follow_ups_due() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 11).expect("date");
        let bill = |id: &str| OpenTask::BillOverdue { document_id: DocumentId::new(id).expect("id"), due: day };
        let follow_up = OpenTask::FollowUpDue {
            case_id: CaseId::new("case-1").expect("id"),
            follow_up_id: FollowUpId::new("f-1").expect("id"),
            date: day,
        };
        let other = OpenTask::EmergencyFolderRefresh { due: day };

        assert_eq!(organize_badges(&[bill("a"), other, follow_up, bill("b")]), OrganizeBadges { to_pay: 2, cases: 1 });
        assert_eq!(organize_badges(&[]), OrganizeBadges::default());
    }
}
