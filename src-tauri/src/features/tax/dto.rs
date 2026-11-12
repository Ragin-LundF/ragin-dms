//! The IPC types of the tax screen and of the tax parts of other screens; names start with the area.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TaxAreaDto {
    Income,
    PensionInsurance,
    HouseholdServices,
    Donations,
    HealthCosts,
    WorkExpenses,
    CapitalIncome,
    Rental,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TaxItemStateDto {
    Found,
    Expected,
    Missing,
    NotNeeded,
}

/// An area of the tax return with its English label and the form it goes into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxAreaOptionDto {
    pub area: TaxAreaDto,
    pub label: String,
    pub form: String,
}

/// The "Tax years" screen for one year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxYearViewDto {
    pub year: i32,
    /// The years the picker offers, newest first.
    pub years: Vec<i32>,
    /// `false` while the checklist of the year was never stored: the screen shows it as a preview and offers "Start"
    /// (`tax_open_year`); no item actions until then. Looking at a year never writes the catalog.
    pub opened: bool,
    pub found: u32,
    /// Every item except the ones not needed this year.
    pub total: u32,
    pub areas: Vec<TaxAreaGroupDto>,
    pub sums: Vec<TaxSumDto>,
    pub area_options: Vec<TaxAreaOptionDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TaxAreaStatusDto {
    Complete,
    Expected,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxAreaGroupDto {
    pub area: TaxAreaOptionDto,
    pub found: u32,
    pub total: u32,
    pub status: TaxAreaStatusDto,
    pub items: Vec<TaxItemDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxItemDto {
    pub id: String,
    pub label: String,
    pub correspondent: Option<String>,
    pub state: TaxItemStateDto,
    pub document_id: Option<String>,
    /// The found document's title or file name, or the item's note.
    pub detail: Option<String>,
    /// The deductible part of the found document, formatted, such as `€ 684.50`.
    pub amount: Option<String>,
    pub added_by_user: bool,
}

/// The deductible parts of an area added up, one text per currency, such as `€ 1,312.40`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxSumDto {
    pub area: TaxAreaOptionDto,
    pub amounts: Vec<String>,
}

/// A change to one item of a year's checklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TaxItemChangeDto {
    Add {
        area: TaxAreaDto,
        label: String,
        correspondent: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Rename {
        item_id: String,
        label: String,
    },
    #[serde(rename_all = "camelCase")]
    SetNotNeeded {
        item_id: String,
        not_needed: bool,
    },
    #[serde(rename_all = "camelCase")]
    LinkDocument {
        item_id: String,
        document_id: String,
    },
}

/// Assigns documents to a tax year and area, or with `year` and `area` both null removes their assignment.
/// `keepDeductible` keeps each document's own deductible part (the bulk bar); otherwise `deductible` (such as
/// `684.50 EUR`, blank for none) is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxAssignInputDto {
    pub document_ids: Vec<String>,
    pub year: Option<i32>,
    pub area: Option<TaxAreaDto>,
    pub deductible: Option<String>,
    pub keep_deductible: bool,
}

/// The tax row of the quick look and what its edit dialog offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxDocumentDto {
    /// "2025 · Household services and craftsmen · € 684.50", or null without an assignment.
    pub text: Option<String>,
    pub year: Option<i32>,
    pub area: Option<TaxAreaDto>,
    /// The deductible part set on the assignment, as the edit field shows it (`684.50 EUR`).
    pub deductible: Option<String>,
    /// The year the edit dialog starts with when there is no assignment yet.
    pub default_year: i32,
    pub area_options: Vec<TaxAreaOptionDto>,
}

/// A document "I have it" may link, best candidates first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxDocumentChoiceDto {
    pub id: String,
    pub name: String,
    pub correspondent: Option<String>,
    pub date: Option<String>,
}

/// The dashboard card "Tax year progress".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaxProgressDto {
    pub year: i32,
    pub found: u32,
    pub total: u32,
    pub missing: u32,
}
