//! The IPC types of the contracts screen; names start with the area. Outputs only serialize and inputs only
//! deserialize, so their names stay as they are in the bindings.

use serde::{Deserialize, Serialize};
use specta::Type;

/// The contracts screen: the totals per currency and one card per contract, running ones first.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractsViewDto {
    pub totals: Vec<ContractTotalDto>,
    pub cards: Vec<ContractCardDto>,
}

/// What the running contracts cost in one currency, such as `312.47 EUR`; `change` is "+ 23.10 EUR / month vs.
/// last year", `None` without a change.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractTotalDto {
    pub monthly: String,
    pub yearly: String,
    pub change: Option<String>,
    pub rising: bool,
}

/// One contract. `color` is the chart colour of its logo letter in the fixed order (0 to 3), 4 for the neutral one
/// once the four are used up.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractCardDto {
    pub id: String,
    pub name: String,
    pub correspondent: Option<String>,
    pub letter: String,
    pub color: u32,
    /// The price per month, such as `44.99 EUR`.
    pub monthly: Option<String>,
    pub pills: Vec<ContractPillDto>,
    pub running: bool,
    pub detail: ContractDetailDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ContractPillTone {
    Warning,
    Danger,
    Neutral,
    Good,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractPillDto {
    pub label: String,
    pub tone: ContractPillTone,
}

/// The detail panel of a contract and what its actions need.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractDetailDto {
    /// "Net Provider · 7 documents".
    pub subtitle: String,
    pub monthly: Option<String>,
    pub yearly: Option<String>,
    /// "1 month", "Cancellable any time" or "Not entered".
    pub notice: String,
    pub cancel_by: Option<String>,
    pub term_end: Option<String>,
    pub ended_on: Option<String>,
    /// The price history, oldest first.
    pub history: Vec<ContractHistoryPointDto>,
    /// "+12 % since 2022" when the monthly price rose over the history.
    pub increase: Option<String>,
    pub note: Option<String>,
    pub document_ids: Vec<String>,
    /// The correspondent "Show documents" filters the library by; `None` hides the button.
    pub documents_filter: Option<String>,
    /// "Show 7 documents": how many documents of that correspondent the library shows.
    pub documents_label: String,
    /// The facts and the recipient of a cancellation letter.
    pub letter_facts: Vec<String>,
    pub recipient: Option<String>,
    pub editor: ContractEditorDto,
}

/// One price of the history: `label` is the amount per month (`44.99`), `cents` its height in the chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractHistoryPointDto {
    pub valid_from: String,
    pub label: String,
    pub cents: u32,
    pub rose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ContractKindDto {
    Utility,
    Telecom,
    Insurance,
    Subscription,
    Membership,
    Loan,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ContractPeriodDto {
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ContractNoticeUnitDto {
    Days,
    Weeks,
    Months,
}

/// The editor's form, as typed: `id` empty for a new contract, `price` such as `44,99` or `44.99 EUR` (the
/// library's currency when none is typed), dates `YYYY-MM-DD`, counts as digits; empty texts mean "not entered".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractEditorDto {
    pub id: String,
    pub name: String,
    pub correspondent: String,
    pub kind: ContractKindDto,
    pub price: String,
    pub period: ContractPeriodDto,
    pub price_valid_from: String,
    pub term_end: String,
    pub renews_by_months: String,
    pub notice_count: String,
    pub notice_unit: ContractNoticeUnitDto,
    pub cancellable_any_time: bool,
    pub note: String,
    pub document_ids: Vec<String>,
}

/// What the AI read from a contract's documents, for the review.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractTermsReviewDto {
    pub prices: Vec<ContractSuggestedPriceDto>,
    /// "Notice 1 month · ends 2026-12-31 · renews by 12 months"; `None` when the AI found no terms.
    pub terms_label: Option<String>,
    pub terms: Option<ContractTermsInputDto>,
    /// "Read by AI (model)".
    pub origin: String,
}

/// One suggested price: "44.99 EUR / month from 2026-01-01", `rose` when it costs more than the price before it.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractSuggestedPriceDto {
    pub label: String,
    pub source: Option<String>,
    pub rose: bool,
    pub point: ContractPricePointDto,
}

/// A price point to accept; `document_id` empty when no document names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractPricePointDto {
    pub valid_from: String,
    pub price: String,
    pub period: ContractPeriodDto,
    pub document_id: String,
}

/// Terms to accept; empty texts keep what the contract has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractTermsInputDto {
    pub term_end: String,
    pub renews_by_months: String,
    pub notice_count: String,
    pub notice_unit: ContractNoticeUnitDto,
}

/// The status card "Contracts per month"; `None` (no card) while no running contract has a price.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractsStatusDto {
    pub monthly: Vec<String>,
    pub increases: u32,
    /// "2 price increases", `None` without one.
    pub increases_label: Option<String>,
}

/// The contracts of one document for the quick look, and the running contracts it could be added to.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractsOfDocumentDto {
    pub contracts: Vec<ContractChoiceDto>,
    pub others: Vec<ContractChoiceDto>,
    pub correspondent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContractChoiceDto {
    pub id: String,
    pub name: String,
}
