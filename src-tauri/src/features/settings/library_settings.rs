//! The settings that live in the catalog: inbox, custom fields, default currency, embedded PDF metadata and the
//! matching rules (Kotlin `LibrarySettingsController`, `FieldsCard`, `RulesCard`, `InboxCard`).

use dms_domain::catalog::Catalog;
use dms_domain::field::{CurrencyCode, FieldDefinition, FieldKey, FieldPresets, FieldRole, FieldType};
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::rule::{MatchingRule, MatchingRuleId, RuleCondition, RuleOrigin};
use dms_domain::settings::{CatalogSettings, InboxSettings};
use dms_domain::text;

use super::dto::{
    FieldRoleChoiceDto, FieldRoleDto, FieldRowDto, FieldTypeChoiceDto, FieldTypeDto, InboxSettingsDto,
    LibrarySettingsChangeDto, LibrarySettingsDto, RuleRowDto,
};
use crate::dto::IpcError;
use crate::messages;
use crate::session::UiMessage;

const FIELD_TYPES: [FieldTypeDto; 5] =
    [FieldTypeDto::Text, FieldTypeDto::Number, FieldTypeDto::Money, FieldTypeDto::Date, FieldTypeDto::Boolean];
const FIELD_ROLES: [FieldRoleDto; 4] =
    [FieldRoleDto::DueDate, FieldRoleDto::CancelBy, FieldRoleDto::ContractEnd, FieldRoleDto::WarrantyEnd];

pub fn present_library_settings(catalog: &Catalog) -> LibrarySettingsDto {
    let settings = catalog.settings();
    let layout = &settings.layout;
    LibrarySettingsDto {
        inbox: inbox_dto(&layout.inbox),
        fields: settings.field_definitions.iter().map(field_row).collect(),
        field_types: FIELD_TYPES
            .iter()
            .map(|value| FieldTypeChoiceDto { value: *value, label: type_name(field_type(*value)).to_owned() })
            .collect(),
        field_roles: FIELD_ROLES
            .iter()
            .map(|value| FieldRoleChoiceDto { value: *value, label: role_name(field_role(*value)).to_owned() })
            .collect(),
        default_currency: layout.default_currency.as_ref().map(|code| code.value().to_owned()).unwrap_or_default(),
        embed_metadata: layout.embed_metadata,
        rules: settings.rules.iter().map(|rule| rule_row(rule, catalog)).collect(),
        owner: super::owner::owner_dto(settings.owner.as_ref()),
    }
}

fn inbox_dto(inbox: &InboxSettings) -> InboxSettingsDto {
    InboxSettingsDto {
        enabled: inbox.enabled,
        path: inbox.path.value().to_owned(),
        ai_analysis: inbox.ai_analysis,
        ai_target_folder: inbox.ai_target_folder,
        auto_analyze: inbox.auto_analyze,
        convert_emails: inbox.convert_emails,
        keep_email_originals: inbox.keep_email_originals,
        propose_splits: inbox.propose_splits,
    }
}

fn field_row(definition: &FieldDefinition) -> FieldRowDto {
    let role = definition.role.map(|role| format!(" · deadline: {}", role_name(role))).unwrap_or_default();
    FieldRowDto {
        key: definition.key.value().to_owned(),
        label: definition.label.clone(),
        text: format!("{}  ({}, {}{role})", definition.label, definition.key.value(), type_name(definition.field_type)),
    }
}

fn rule_row(rule: &MatchingRule, catalog: &Catalog) -> RuleRowDto {
    let origin = match rule.origin {
        RuleOrigin::Learned => "learned",
        RuleOrigin::Manual => "manual",
    };
    RuleRowDto {
        id: rule.id.value().to_owned(),
        name: rule.name.clone(),
        title: format!("{} ({origin})", rule.name),
        description: describe_rule(rule, catalog),
        enabled: rule.enabled,
    }
}

fn describe_rule(rule: &MatchingRule, catalog: &Catalog) -> String {
    let conditions: Vec<String> = rule
        .conditions
        .iter()
        .map(|condition| match condition {
            RuleCondition::CorrespondentIs { name } => format!("from {name}"),
            RuleCondition::TextContains { words, require_all } => {
                format!("text has {} {}", if *require_all { "all of" } else { "any of" }, words.join(", "))
            }
            RuleCondition::FileNameMatches { glob } => format!("name like {glob}"),
        })
        .collect();
    let actions = &rule.actions;
    let category = actions.category_id.as_ref().map(|id| {
        let name = catalog.categories().get(id).map_or_else(|| id.value(), |category| category.name.as_str());
        format!("category {name}")
    });
    let tags = (!actions.tags.is_empty()).then(|| {
        format!("tags {}", actions.tags.iter().map(dms_domain::taxonomy::TagName::value).collect::<Vec<_>>().join(", "))
    });
    let fields = (!actions.fields.is_empty()).then(|| format!("{} field values", actions.fields.len()));
    let folder = actions.target_folder.as_ref().map(|folder| format!("folder {}", folder.value()));
    let actions: Vec<String> = [category, tags, fields, folder].into_iter().flatten().collect();
    format!("When {}: {}", conditions.join(" and "), actions.join(", "))
}

fn type_name(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::Text => "Text",
        FieldType::Number => "Number",
        FieldType::Money => "Amount",
        FieldType::Date => "Date",
        FieldType::Boolean => "Yes/no",
    }
}

fn role_name(role: FieldRole) -> &'static str {
    match role {
        FieldRole::DueDate => "Due date",
        FieldRole::CancelBy => "Cancel by",
        FieldRole::ContractEnd => "Contract end",
        FieldRole::WarrantyEnd => "Warranty end",
    }
}

const fn field_type(dto: FieldTypeDto) -> FieldType {
    match dto {
        FieldTypeDto::Text => FieldType::Text,
        FieldTypeDto::Number => FieldType::Number,
        FieldTypeDto::Money => FieldType::Money,
        FieldTypeDto::Date => FieldType::Date,
        FieldTypeDto::Boolean => FieldType::Boolean,
    }
}

const fn field_role(dto: FieldRoleDto) -> FieldRole {
    match dto {
        FieldRoleDto::DueDate => FieldRole::DueDate,
        FieldRoleDto::CancelBy => FieldRole::CancelBy,
        FieldRoleDto::ContractEnd => FieldRole::ContractEnd,
        FieldRoleDto::WarrantyEnd => FieldRole::WarrantyEnd,
    }
}

/// A catalog-settings change ready for the use case: `apply` runs on the newest settings inside it, `subject`
/// names it in the activity log and `done` is the message on success.
pub struct SettingsChange {
    pub subject: String,
    pub done: String,
    pub apply: Box<dyn Fn(&CatalogSettings) -> CatalogSettings + Send + Sync>,
}

/// What a change request comes to: a change, a message why it cannot be made, or nothing (the rule is gone).
pub enum SettingsPlan {
    Change(SettingsChange),
    Refused(UiMessage),
    Nothing,
}

/// Turns what the user did into a change of `current`'s settings.
///
/// # Errors
/// `invalid_input` for a blank field key or rule id.
pub fn plan_change(change: LibrarySettingsChangeDto, current: &CatalogSettings) -> Result<SettingsPlan, IpcError> {
    let invalid = |error: dms_domain::DomainError| IpcError::invalid(error.to_string());
    let plan =
        |subject: String, done: String, apply: Box<dyn Fn(&CatalogSettings) -> CatalogSettings + Send + Sync>| {
            Ok(SettingsPlan::Change(SettingsChange { subject, done, apply }))
        };
    match change {
        LibrarySettingsChangeDto::AddField { label, field_type: chosen, role } => {
            Ok(plan_new_field(&label, field_type(chosen), role.map(field_role), current))
        }
        LibrarySettingsChangeDto::RemoveField { key } => {
            let key = FieldKey::new(key).map_err(invalid)?;
            plan(
                format!("Field {}", key.value()),
                "Removed the field".to_owned(),
                Box::new(move |settings| settings.without_field(&key)),
            )
        }
        LibrarySettingsChangeDto::AddStandardFields => {
            plan("Standard fields".to_owned(), "Added the standard fields".to_owned(), Box::new(with_standard_fields))
        }
        LibrarySettingsChangeDto::SaveCurrency { typed } => {
            let currency = Some(text::trim(&typed)).filter(|code| !code.is_empty()).map(CurrencyCode::parse);
            let Some(currency) = currency.map_or(Some(None), |parsed| parsed.map(Some)) else {
                return Ok(SettingsPlan::Refused(UiMessage::error("Write a three-letter currency code such as EUR")));
            };
            plan(
                "Default currency".to_owned(),
                "Saved the default currency".to_owned(),
                Box::new(move |settings| {
                    let layout = dms_domain::settings::LibraryLayout {
                        default_currency: currency.clone(),
                        ..settings.layout.clone()
                    };
                    settings.with_layout(layout)
                }),
            )
        }
        LibrarySettingsChangeDto::SetEmbedMetadata { enabled } => plan(
            "Embedded PDF metadata".to_owned(),
            if enabled { "PDFs get the approved metadata" } else { "PDFs stay as they are" }.to_owned(),
            Box::new(move |settings| {
                settings.with_layout(dms_domain::settings::LibraryLayout {
                    embed_metadata: enabled,
                    ..settings.layout.clone()
                })
            }),
        ),
        LibrarySettingsChangeDto::SetRuleEnabled { rule_id, enabled } => {
            let id = MatchingRuleId::new(rule_id).map_err(invalid)?;
            let Some(rule) = current.rules.iter().find(|rule| rule.id == id) else {
                return Ok(SettingsPlan::Nothing);
            };
            let switched = MatchingRule { enabled, ..rule.clone() };
            plan(
                format!("Rule {}", rule.name),
                format!("Switched {} {}", rule.name, if enabled { "on" } else { "off" }),
                Box::new(move |settings| settings.with_rule(switched.clone())),
            )
        }
        LibrarySettingsChangeDto::RemoveRule { rule_id } => {
            let id = MatchingRuleId::new(rule_id).map_err(invalid)?;
            plan("Rule".to_owned(), "Removed the rule".to_owned(), Box::new(move |settings| settings.without_rule(&id)))
        }
        LibrarySettingsChangeDto::SaveOwner { owner } => Ok(super::owner::plan_owner(&owner)),
    }
}

/// A field named `label`; its key comes from the label and must be new. The role is kept only for dates.
fn plan_new_field(label: &str, chosen: FieldType, role: Option<FieldRole>, current: &CatalogSettings) -> SettingsPlan {
    let Some(key) = FieldKey::from_label(label).filter(|key| current.field(key).is_none()) else {
        return SettingsPlan::Refused(UiMessage::error("Choose another name for the field"));
    };
    let definition = FieldDefinition {
        key,
        label: text::trim(label).to_owned(),
        field_type: chosen,
        role: role.filter(|_| chosen == FieldType::Date),
    };
    SettingsPlan::Change(SettingsChange {
        subject: format!("Field {}", definition.label),
        done: format!("Added the field {}", definition.label),
        apply: Box::new(move |settings| settings.with_field(definition.clone())),
    })
}

/// Adds the presets whose key is missing, in their order; existing definitions stay untouched.
fn with_standard_fields(settings: &CatalogSettings) -> CatalogSettings {
    FieldPresets::all().into_iter().fold(settings.clone(), |current, preset| {
        if current.field(&preset.key).is_none() { current.with_field(preset) } else { current }
    })
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/').map(text::trim).filter(|segment| !segment.is_empty()).collect()
}

/// Why `path` cannot be the inbox, or `None` if it can.
pub fn inbox_path_problem(path: &str) -> Option<&'static str> {
    let segments = path_segments(path);
    if segments.is_empty() {
        return Some("Name a folder inside the library, such as Inbox");
    }
    segments.into_iter().find_map(FileNamePolicy::check).map(messages::name_problem)
}

/// The inbox settings the user typed; `None` while the path has a problem.
pub fn inbox_settings_of(draft: &InboxSettingsDto) -> Option<InboxSettings> {
    if inbox_path_problem(&draft.path).is_some() {
        return None;
    }
    let path = RelativePath::from_segments(&path_segments(&draft.path)).ok()?;
    Some(InboxSettings {
        enabled: draft.enabled,
        path,
        ai_analysis: draft.ai_analysis,
        ai_target_folder: draft.ai_target_folder,
        auto_analyze: draft.auto_analyze,
        convert_emails: draft.convert_emails,
        keep_email_originals: draft.keep_email_originals,
        propose_splits: draft.propose_splits,
    })
}

/// "The inbox is {path} in every library folder" or "The inbox is off".
pub fn inbox_saved_message(inbox: &InboxSettings) -> String {
    if inbox.enabled {
        format!("The inbox is {} in every library folder", inbox.path)
    } else {
        "The inbox is off".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use dms_domain::catalog::{CatalogUpdate, LibraryId};
    use dms_domain::rule::RuleActions;
    use dms_domain::taxonomy::{Category, CategoryId, TagName};

    use super::*;

    fn applied(update: Result<CatalogUpdate, dms_domain::DomainError>) -> Catalog {
        match update.expect("valid change") {
            CatalogUpdate::Applied(catalog) => catalog,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    }

    fn rule(id: &str, conditions: Vec<RuleCondition>, actions: RuleActions, origin: RuleOrigin) -> MatchingRule {
        MatchingRule::new(MatchingRuleId::new(id).expect("id"), format!("Rule {id}"), conditions, actions, origin)
            .expect("rule")
    }

    fn catalog_with(settings: CatalogSettings) -> Catalog {
        let catalog = Catalog::empty(LibraryId::new("library-1").expect("id"));
        let category = Category::new(CategoryId::new("c-bills").expect("id"), "Bills", None, None).expect("category");
        let catalog = applied(catalog.add_category(category));
        applied(catalog.change_settings(settings))
    }

    fn change(plan: SettingsPlan, settings: &CatalogSettings) -> (String, String, CatalogSettings) {
        match plan {
            SettingsPlan::Change(change) => (change.subject, change.done, (change.apply)(settings)),
            SettingsPlan::Refused(message) => panic!("refused: {}", message.text),
            SettingsPlan::Nothing => panic!("nothing to change"),
        }
    }

    #[test]
    fn field_rows_name_type_and_deadline_role() {
        let settings = CatalogSettings {
            field_definitions: vec![FieldPresets::amount(), FieldPresets::due_date()],
            ..CatalogSettings::default()
        };
        let dto = present_library_settings(&catalog_with(settings));
        let texts: Vec<_> = dto.fields.iter().map(|row| row.text.as_str()).collect();
        assert_eq!(texts, vec!["Amount  (amount, Amount)", "Due date  (due, Date · deadline: Due date)"]);
        assert_eq!(dto.field_types[4].label, "Yes/no");
        assert_eq!(dto.default_currency, "");
    }

    #[test]
    fn rule_rows_describe_conditions_and_actions() {
        let actions = RuleActions {
            category_id: Some(CategoryId::new("c-bills").expect("id")),
            tags: [TagName::of("invoice").expect("tag"), TagName::of("home").expect("tag")].into_iter().collect(),
            target_folder: Some(RelativePath::of("Home/Bills").expect("path")),
            ..RuleActions::default()
        };
        let conditions = vec![
            RuleCondition::CorrespondentIs { name: "City Power".to_owned() },
            RuleCondition::TextContains { words: vec!["meter".to_owned(), "kWh".to_owned()], require_all: true },
            RuleCondition::FileNameMatches { glob: "*.pdf".to_owned() },
        ];
        let unknown =
            RuleActions { category_id: Some(CategoryId::new("c-gone").expect("id")), ..RuleActions::default() };
        let settings = CatalogSettings {
            rules: vec![
                rule("r1", conditions, actions, RuleOrigin::Learned),
                rule("r2", vec![RuleCondition::FileNameMatches { glob: "x?".to_owned() }], unknown, RuleOrigin::Manual),
            ],
            ..CatalogSettings::default()
        };
        let rows = present_library_settings(&catalog_with(settings)).rules;
        assert_eq!(rows[0].title, "Rule r1 (learned)");
        assert_eq!(
            rows[0].description,
            "When from City Power and text has all of meter, kWh and name like *.pdf: category Bills, tags invoice, home, folder Home/Bills"
        );
        assert_eq!(rows[1].description, "When name like x?: category c-gone");
    }

    #[test]
    fn a_new_field_needs_a_usable_new_key_and_keeps_the_role_only_for_dates() {
        let current = CatalogSettings { field_definitions: vec![FieldPresets::amount()], ..CatalogSettings::default() };
        let taken = plan_new_field(" Amount ", FieldType::Money, None, &current);
        assert!(matches!(taken, SettingsPlan::Refused(message) if message.text == "Choose another name for the field"));
        assert!(matches!(plan_new_field("!!", FieldType::Text, None, &current), SettingsPlan::Refused(_)));
        let (subject, done, saved) =
            change(plan_new_field(" Paid on ", FieldType::Text, Some(FieldRole::DueDate), &current), &current);
        assert_eq!((subject.as_str(), done.as_str()), ("Field Paid on", "Added the field Paid on"));
        let added = &saved.field_definitions[1];
        assert_eq!((added.key.value(), added.label.as_str(), added.role), ("paid-on", "Paid on", None));
    }

    #[test]
    fn standard_fields_add_only_missing_presets_in_order() {
        let mine = FieldDefinition { label: "Total".to_owned(), ..FieldPresets::amount() };
        let current = CatalogSettings { field_definitions: vec![mine.clone()], ..CatalogSettings::default() };
        let (_, done, saved) =
            change(plan_change(LibrarySettingsChangeDto::AddStandardFields, &current).expect("plan"), &current);
        assert_eq!(done, "Added the standard fields");
        let keys: Vec<_> = saved.field_definitions.iter().map(|field| field.key.value()).collect();
        assert_eq!(keys, vec!["amount", "due", "cancel-by", "contract-end", "warranty-until"]);
        assert_eq!(saved.field_definitions[0], mine);
    }

    #[test]
    fn currency_is_parsed_uppercase_empty_removes_it_and_invalid_is_refused() {
        let current = CatalogSettings::default();
        let save = |typed: &str| {
            plan_change(LibrarySettingsChangeDto::SaveCurrency { typed: typed.to_owned() }, &current).expect("plan")
        };
        let (_, _, saved) = change(save(" eur "), &current);
        assert_eq!(saved.layout.default_currency.as_ref().map(CurrencyCode::value), Some("EUR"));
        let (_, _, cleared) = change(save(" "), &saved);
        assert_eq!(cleared.layout.default_currency, None);
        assert!(
            matches!(save("EURO"), SettingsPlan::Refused(message) if message.text == "Write a three-letter currency code such as EUR")
        );
    }

    #[test]
    fn switching_a_rule_names_it_and_a_missing_rule_changes_nothing() {
        let current = CatalogSettings {
            rules: vec![rule(
                "r1",
                vec![RuleCondition::FileNameMatches { glob: "*".to_owned() }],
                RuleActions { target_folder: Some(RelativePath::of("A").expect("path")), ..RuleActions::default() },
                RuleOrigin::Manual,
            )],
            ..CatalogSettings::default()
        };
        let switch = |id: &str| LibrarySettingsChangeDto::SetRuleEnabled { rule_id: id.to_owned(), enabled: false };
        let (_, done, saved) = change(plan_change(switch("r1"), &current).expect("plan"), &current);
        assert_eq!(done, "Switched Rule r1 off");
        assert!(!saved.rules[0].enabled);
        assert!(matches!(plan_change(switch("gone"), &current).expect("plan"), SettingsPlan::Nothing));
    }

    #[test]
    fn inbox_path_problems_name_the_first_broken_segment() {
        assert_eq!(inbox_path_problem(" / "), Some("Name a folder inside the library, such as Inbox"));
        assert_eq!(inbox_path_problem("Inbox/.dms-text"), Some("The name must not start with a dot or a space"));
        assert_eq!(inbox_path_problem(" Mail / Inbox "), None);
        let draft = InboxSettingsDto {
            enabled: true,
            path: " Mail / Inbox ".to_owned(),
            ai_analysis: true,
            ai_target_folder: false,
            auto_analyze: false,
            convert_emails: true,
            keep_email_originals: false,
            propose_splits: true,
        };
        let inbox = inbox_settings_of(&draft).expect("valid");
        assert_eq!(inbox.path.value(), "Mail/Inbox");
        assert!(inbox.convert_emails && !inbox.keep_email_originals);
        assert!(inbox.propose_splits);
        assert_eq!(inbox_saved_message(&inbox), "The inbox is Mail/Inbox in every library folder");
    }
}
