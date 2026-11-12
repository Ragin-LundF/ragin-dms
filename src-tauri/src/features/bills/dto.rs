//! The IPC types of the bills screen; names start with the area.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Which bills the "To pay" list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BillsFilterDto {
    Open,
    Paid,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BillStatusDto {
    Open,
    Overdue,
    Paid,
}

/// Where a bill's values came from: an e-invoice in the PDF, an approved AI suggestion, a matching rule, or typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BillSourceDto {
    EInvoice,
    Ai,
    Rule,
    Typed,
}

/// A sum of money per currency, already written as text ("86.40 EUR", several joined by " + "), and how many bills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BillsTotalDto {
    pub amount: String,
    pub count: u32,
}

/// One bill. `due` is `YYYY-MM-DD`, `due_in_days` negative once it is overdue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BillRowDto {
    pub document_id: String,
    pub title: String,
    pub payee: String,
    pub due: String,
    pub due_in_days: i32,
    pub status: BillStatusDto,
    pub paid_on: Option<String>,
    pub amount: String,
    pub source: BillSourceDto,
    /// Why no payment code can be shown, such as a missing IBAN; `null` when it can.
    pub qr_refusal: Option<String>,
}

/// The "To pay" screen: totals, the bills of the filter by due date, and the older bills never marked.
/// `tracking_started` is false until the screen was opened once; the screen then starts tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BillsViewDto {
    pub open: BillsTotalDto,
    pub overdue: BillsTotalDto,
    pub paid_this_month: BillsTotalDto,
    /// Unpaid bills due by the end of this month, overdue ones included (the dashboard card).
    pub due_this_month: BillsTotalDto,
    pub month: String,
    pub rows: Vec<BillRowDto>,
    pub older: Vec<BillRowDto>,
    pub tracking_started: bool,
}

/// What the payment dialog shows: the QR code as SVG markup and the values it holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BillPaymentCodeDto {
    pub document_id: String,
    pub title: String,
    pub svg: Option<String>,
    pub refusal: Option<String>,
    pub payee: String,
    pub iban: Option<String>,
    pub amount: String,
    pub reference: Option<String>,
    pub due: String,
    pub source: BillSourceDto,
    pub paid: bool,
}

/// The bill row of the quick look; `is_bill` false for other documents, `can_read_e_invoice` for PDFs, which may carry an e-invoice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BillDocumentDto {
    pub is_bill: bool,
    pub status: Option<BillStatusDto>,
    pub paid_on: Option<String>,
    pub can_read_e_invoice: bool,
}

/// One "expected but missing" document of the dashboard card. `rhythm` is in words ("Arrived every January since
/// 2021"), `expected_on` `YYYY-MM-DD`; `document_ids` are the sender's latest documents, for a request letter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedDocumentDto {
    pub task_key: String,
    pub title: String,
    pub correspondent: String,
    pub rhythm: String,
    pub expected_on: String,
    pub document_ids: Vec<String>,
}
