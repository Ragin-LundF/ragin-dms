use dms_application::port::assistant::{QueryAnswer, QueryRequest, QueryTranslator, TranslatedQuery};

use crate::support::Shared;

/// Answers with `answer` and records the requests it was given.
pub struct FakeQueryTranslator {
    pub answer: Shared<QueryAnswer>,
    pub requests: Shared<Vec<QueryRequest>>,
}

impl FakeQueryTranslator {
    pub const MODEL: &'static str = "query-model";

    pub fn answer_with(&self, query: TranslatedQuery) {
        self.answer.set(QueryAnswer::Translated { query, model: Self::MODEL.to_owned() });
    }
}

impl Default for FakeQueryTranslator {
    fn default() -> Self {
        Self {
            answer: Shared::new(QueryAnswer::Translated {
                query: TranslatedQuery::default(),
                model: Self::MODEL.to_owned(),
            }),
            requests: Shared::default(),
        }
    }
}

impl QueryTranslator for FakeQueryTranslator {
    fn translate(&self, request: &QueryRequest) -> QueryAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.answer.get()
    }
}
