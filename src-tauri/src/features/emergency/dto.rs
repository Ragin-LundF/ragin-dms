//! The IPC types of the emergency screen; names start with the area.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum EmergencyGroupDto {
    Contacts,
    Insurance,
    Accounts,
    Contracts,
    Personal,
    Other,
}

/// The emergency screen: the refresh status, the items by group, the overview as it will be printed, and what can
/// still be added (contracts not in the folder yet) or chosen (folders to save the overview in).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyStateDto {
    pub status: EmergencyStatusDto,
    pub refresh_months: u8,
    pub intro: String,
    pub groups: Vec<EmergencyGroupListDto>,
    pub preview: EmergencyPreviewDto,
    pub included: u32,
    pub contracts: Vec<EmergencyChoiceDto>,
    pub folders: Vec<String>,
}

/// "Last refreshed …" or "Not refreshed yet", the line below it and whether the refresh is due (the reminder runs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyStatusDto {
    pub title: String,
    pub detail: String,
    pub refresh_due: bool,
    pub changed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyGroupListDto {
    pub group: EmergencyGroupDto,
    pub label: String,
    pub items: Vec<EmergencyItemDto>,
}

/// One item: its line of text, whether it goes into the overview and the ZIP, and whether it changed since the last
/// refresh. `document_id` lets the window open a document item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyItemDto {
    pub id: String,
    pub kind: EmergencyItemKindDto,
    pub text: String,
    pub included: bool,
    pub changed: bool,
    pub document_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum EmergencyItemKindDto {
    Document,
    Contract,
    Contact,
    Note,
}

/// The first page of the overview: title, owner and date, intro and the table rows of the included items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyPreviewDto {
    pub title: String,
    pub subtitle: String,
    pub intro: String,
    pub rows: Vec<EmergencyPreviewRowDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyPreviewRowDto {
    pub area: String,
    pub text: String,
}

/// A document or contract that can be added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyChoiceDto {
    pub id: String,
    pub name: String,
    pub detail: String,
}

/// A change the user made on the screen, in the quick look or in the contract detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EmergencyEditDto {
    #[serde(rename_all = "camelCase")]
    AddDocument {
        group: EmergencyGroupDto,
        document_id: String,
    },
    #[serde(rename_all = "camelCase")]
    AddContract {
        group: EmergencyGroupDto,
        contract_id: String,
    },
    #[serde(rename_all = "camelCase")]
    AddContact {
        group: EmergencyGroupDto,
        name: String,
        role: String,
        phone: String,
        email: String,
        note: String,
    },
    #[serde(rename_all = "camelCase")]
    AddNote {
        group: EmergencyGroupDto,
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Remove {
        item_id: String,
    },
    #[serde(rename_all = "camelCase")]
    SetIncluded {
        item_id: String,
        included: bool,
    },
    SetIntro {
        intro: String,
    },
    SetRefreshMonths {
        months: u8,
    },
}

/// Where the overview goes: a library folder, or a file outside the library chosen in the save dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EmergencyOverviewTargetDto {
    Library { folder: String },
    Outside { path: String },
}

/// Whether a document or contract is in the emergency folder, for the quick look and the contract detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyMembershipDto {
    pub item_id: Option<String>,
    pub group: Option<String>,
}
