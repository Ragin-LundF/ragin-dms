use serde::{Deserialize, Serialize};
use specta::Type;

/// The Settings screen's device part: the library folders of this device and its preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsStateDto {
    /// Whether a primary folder is configured; without one the screen shows the welcome card.
    pub configured: bool,
    pub folders: Vec<FolderRowDto>,
    pub add_new_files_while_scanning: bool,
    pub ai: AiSettingsDto,
}

/// One configured library folder; `note` says why the tool currently leaves it alone, if it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FolderRowDto {
    pub id: String,
    pub display_name: String,
    pub location: String,
    pub primary: bool,
    pub note: Option<String>,
}

/// A change to the library folders; a chosen `location` is an absolute path of this device.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryCommandDto {
    CreatePrimary {
        location: String,
    },
    AddSyncFolder {
        location: String,
    },
    #[serde(rename_all = "camelCase")]
    RemoveSyncFolder {
        folder_id: String,
    },
    #[serde(rename_all = "camelCase")]
    ChoosePrimary {
        folder_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderDto {
    Local,
    Anthropic,
}

/// The saved AI settings of this device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)] // independent switches of the settings
pub struct AiSettingsDto {
    pub enabled: bool,
    pub base_url: String,
    pub model: Option<String>,
    pub prefer_vision_for_pdfs: bool,
    pub timeout_seconds: i32,
    pub accept_suggested_file_names: bool,
    pub embedding_model: Option<String>,
    pub input_characters: i32,
    pub provider: AiProviderDto,
    pub anthropic_model: String,
}

/// The AI settings as typed, before they are saved (Kotlin `AiDraftModel`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)] // independent switches of the settings
pub struct AiDraftDto {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub timeout_seconds: String,
    pub prefer_vision_for_pdfs: bool,
    pub accept_suggested_file_names: bool,
    pub embedding_model: String,
    pub input_characters: String,
    pub provider: AiProviderDto,
    pub anthropic_model: String,
}

/// What a draft is worth: which fields are wrong, the first problem, the settings it describes once valid, and
/// why "Save" waits (the problem, or "Nothing changed").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiDraftCheckDto {
    pub address_valid: bool,
    pub timeout_valid: bool,
    pub input_valid: bool,
    pub problem: Option<String>,
    pub settings: Option<AiSettingsDto>,
    pub save_disabled_reason: Option<String>,
}

/// The result of a connection test: the models the server offers, or why it did not answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionDto {
    pub models: Vec<String>,
    pub message: String,
    pub is_error: bool,
}

/// The settings that live in the catalog and travel to every folder and device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettingsDto {
    pub inbox: InboxSettingsDto,
    pub fields: Vec<FieldRowDto>,
    pub field_types: Vec<FieldTypeChoiceDto>,
    pub field_roles: Vec<FieldRoleChoiceDto>,
    /// Empty while none is set.
    pub default_currency: String,
    pub embed_metadata: bool,
    pub rules: Vec<RuleRowDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)] // independent switches of the catalog format
pub struct InboxSettingsDto {
    pub enabled: bool,
    pub path: String,
    pub ai_analysis: bool,
    pub ai_target_folder: bool,
    pub auto_analyze: bool,
}

/// One custom field; `text` is the whole line, such as "Amount  (amount, Amount)".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FieldRowDto {
    pub key: String,
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FieldTypeDto {
    Text,
    Number,
    Money,
    Date,
    Boolean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FieldRoleDto {
    DueDate,
    CancelBy,
    ContractEnd,
    WarrantyEnd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FieldTypeChoiceDto {
    pub value: FieldTypeDto,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FieldRoleChoiceDto {
    pub value: FieldRoleDto,
    pub label: String,
}

/// One matching rule: `title` is "{name} (learned|manual)", `description` what it looks for and proposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuleRowDto {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub enabled: bool,
}

/// One change of the catalog settings, as the Settings cards make them.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibrarySettingsChangeDto {
    #[serde(rename_all = "camelCase")]
    AddField {
        label: String,
        field_type: FieldTypeDto,
        role: Option<FieldRoleDto>,
    },
    RemoveField {
        key: String,
    },
    AddStandardFields,
    SaveCurrency {
        typed: String,
    },
    SetEmbedMetadata {
        enabled: bool,
    },
    #[serde(rename_all = "camelCase")]
    SetRuleEnabled {
        rule_id: String,
        enabled: bool,
    },
    #[serde(rename_all = "camelCase")]
    RemoveRule {
        rule_id: String,
    },
}
