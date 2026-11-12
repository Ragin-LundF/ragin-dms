use std::sync::Arc;

use crate::cases::{
    AddCaseNote, AddFollowUp, AddToCase, CompleteFollowUp, CreateCase, DeleteCase, RemoveFromCase, SetCaseStatus,
    SnoozeFollowUp,
};
use crate::library::CatalogStore;
use crate::port::system::{Clock, IdGenerator};

/// The use cases of cases, wired once.
pub struct CaseServices {
    pub create_case: Arc<CreateCase>,
    pub add_to_case: Arc<AddToCase>,
    pub remove_from_case: Arc<RemoveFromCase>,
    pub set_case_status: Arc<SetCaseStatus>,
    pub add_case_note: Arc<AddCaseNote>,
    pub add_follow_up: Arc<AddFollowUp>,
    pub complete_follow_up: Arc<CompleteFollowUp>,
    pub snooze_follow_up: Arc<SnoozeFollowUp>,
    pub delete_case: Arc<DeleteCase>,
}

impl CaseServices {
    pub fn new(store: &Arc<CatalogStore>, clock: &Arc<dyn Clock>, ids: &Arc<dyn IdGenerator>) -> Self {
        Self {
            create_case: Arc::new(CreateCase::new(store.clone(), clock.clone(), ids.clone())),
            add_to_case: Arc::new(AddToCase::new(store.clone(), clock.clone())),
            remove_from_case: Arc::new(RemoveFromCase::new(store.clone())),
            set_case_status: Arc::new(SetCaseStatus::new(store.clone(), clock.clone())),
            add_case_note: Arc::new(AddCaseNote::new(store.clone(), clock.clone())),
            add_follow_up: Arc::new(AddFollowUp::new(store.clone(), ids.clone())),
            complete_follow_up: Arc::new(CompleteFollowUp::new(store.clone())),
            snooze_follow_up: Arc::new(SnoozeFollowUp::new(store.clone(), clock.clone())),
            delete_case: Arc::new(DeleteCase::new(store.clone())),
        }
    }
}
