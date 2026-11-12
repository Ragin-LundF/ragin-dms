use dms_domain::cases::CaseId;
use dms_domain::catalog::Catalog;

use crate::features::dashboard::dto::{DashboardAction, DashboardButtonDto, DashboardButtonStyle};
use crate::features::dashboard::view::buttons::{ButtonExt, button};

/// A follow-up due on the dashboard: open its case, mark it done, or snooze it.
pub fn follow_up_buttons(case_id: &CaseId, task_key: &str) -> Vec<DashboardButtonDto> {
    vec![
        open_case(case_id),
        button("Done", DashboardAction::CompleteFollowUp { task_key: task_key.to_owned() }).icon("check"),
        button("Snooze 7 days", DashboardAction::SnoozeFollowUp { task_key: task_key.to_owned() })
            .style(DashboardButtonStyle::Text),
    ]
}

/// A case waiting too long: open it, draft a reminder, close it, or dismiss the task until it waits anew.
pub fn waiting_buttons(case_id: &CaseId, task_key: &str, catalog: &Catalog) -> Vec<DashboardButtonDto> {
    let case = catalog.cases().get(case_id);
    let reminder = DashboardAction::DraftReminder {
        case_id: case_id.to_string(),
        document_ids: case
            .into_iter()
            .flat_map(dms_domain::cases::Case::referenced_documents)
            .filter(|id| catalog.documents().contains_key(*id))
            .map(ToString::to_string)
            .collect(),
        recipient: case.and_then(|case| case.correspondent.clone()),
    };
    vec![
        open_case(case_id),
        button("Draft reminder", reminder).icon("edit_note"),
        button("Mark as done", DashboardAction::CloseCase { task_key: task_key.to_owned() }).icon("check"),
        button("Dismiss", DashboardAction::Dismiss { task_key: task_key.to_owned() }).style(DashboardButtonStyle::Text),
    ]
}

fn open_case(case_id: &CaseId) -> DashboardButtonDto {
    button("Open case", DashboardAction::OpenCase { case_id: case_id.to_string() })
        .icon("arrow_forward")
        .style(DashboardButtonStyle::Tonal)
}
