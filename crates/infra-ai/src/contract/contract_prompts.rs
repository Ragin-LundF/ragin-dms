//! The prompts of reading a contract's terms: a fixed system part, and the request with the numbered documents.

use dms_application::port::assistant::ContractTermsRequest;

use crate::prompt;

pub(crate) fn system() -> String {
    prompt::CONTRACT_TERMS.text()
}

pub(crate) fn request(request: &ContractTermsRequest) -> String {
    let documents: Vec<String> =
        request.source_texts.iter().enumerate().map(|(index, text)| format!("[{}] {text}", index + 1)).collect();
    prompt::CONTRACT_TERMS_REQUEST.fill(&[
        ("name", &request.contract_name),
        ("correspondent", request.correspondent.as_deref().unwrap_or("unknown")),
        ("documents", &documents.join("\n\n---\n\n")),
    ])
}
