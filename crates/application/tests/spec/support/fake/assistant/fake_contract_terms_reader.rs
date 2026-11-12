use dms_application::port::assistant::{
    ContractTermsAnswer, ContractTermsReader, ContractTermsReading, ContractTermsRequest,
};

use crate::support::Shared;

/// Answers with `answer` and records the requests it was given.
pub struct FakeContractTermsReader {
    pub answer: Shared<ContractTermsAnswer>,
    pub requests: Shared<Vec<ContractTermsRequest>>,
}

impl FakeContractTermsReader {
    pub const MODEL: &'static str = "terms-model";
}

impl Default for FakeContractTermsReader {
    fn default() -> Self {
        Self {
            answer: Shared::new(ContractTermsAnswer::Read {
                reading: ContractTermsReading::default(),
                model: Self::MODEL.to_owned(),
            }),
            requests: Shared::default(),
        }
    }
}

impl ContractTermsReader for FakeContractTermsReader {
    fn read(&self, request: &ContractTermsRequest) -> ContractTermsAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.answer.get()
    }
}
