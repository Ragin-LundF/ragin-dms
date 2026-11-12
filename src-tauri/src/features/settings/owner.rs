//! The card "Your details (for letters)": the owner's name, address, email and phone, kept in the catalog so every
//! device writes letters with the same sender.

use dms_domain::settings::{CatalogSettings, OwnerDetails};
use dms_domain::text;

use super::dto::SettingsOwnerDto;
use super::library_settings::{SettingsChange, SettingsPlan};
use crate::session::UiMessage;

pub fn owner_dto(owner: Option<&OwnerDetails>) -> SettingsOwnerDto {
    owner.map_or_else(SettingsOwnerDto::default, |owner| SettingsOwnerDto {
        name: owner.name.clone(),
        address: owner.address_lines.join("\n"),
        email: owner.email.clone().unwrap_or_default(),
        phone: owner.phone.clone().unwrap_or_default(),
    })
}

/// The typed details, trimmed, blank address lines left out; all empty removes them, a missing name is refused.
pub fn plan_owner(typed: &SettingsOwnerDto) -> SettingsPlan {
    let owner = owner_of(typed);
    if let Some(problem) = owner.as_ref().and_then(|owner| owner.validate().err()) {
        return SettingsPlan::Refused(UiMessage::error(problem.to_string()));
    }
    let done = if owner.is_some() { "Saved your details" } else { "Removed your details" };
    SettingsPlan::Change(SettingsChange {
        subject: "Your details".to_owned(),
        done: done.to_owned(),
        apply: Box::new(move |settings| CatalogSettings { owner: owner.clone(), ..settings.clone() }),
    })
}

fn owner_of(typed: &SettingsOwnerDto) -> Option<OwnerDetails> {
    let filled = |value: &str| Some(text::trim(value).to_owned()).filter(|value| !value.is_empty());
    let address_lines: Vec<String> = typed.address.lines().filter_map(filled).collect();
    let (name, email, phone) = (filled(&typed.name), filled(&typed.email), filled(&typed.phone));
    if name.is_none() && address_lines.is_empty() && email.is_none() && phone.is_none() {
        return None;
    }
    Some(OwnerDetails { name: name.unwrap_or_default(), address_lines, email, phone })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(name: &str, address: &str) -> SettingsOwnerDto {
        SettingsOwnerDto {
            name: name.to_owned(),
            address: address.to_owned(),
            email: " ".to_owned(),
            phone: String::new(),
        }
    }

    fn applied(plan: SettingsPlan) -> (String, CatalogSettings) {
        match plan {
            SettingsPlan::Change(change) => (change.done, (change.apply)(&CatalogSettings::default())),
            SettingsPlan::Refused(message) => panic!("refused: {}", message.text),
            SettingsPlan::Nothing => panic!("nothing to change"),
        }
    }

    #[test]
    fn typed_details_are_trimmed_and_blank_address_lines_dropped() {
        let (done, settings) = applied(plan_owner(&typed(" Alex Example ", "Main Street 1\n\n  12345 Exampletown \n")));

        assert_eq!(done, "Saved your details");
        let owner = settings.owner.expect("details");
        assert_eq!(owner.name, "Alex Example");
        assert_eq!(owner.address_lines, vec!["Main Street 1".to_owned(), "12345 Exampletown".to_owned()]);
        assert_eq!((owner.email.as_deref(), owner.phone.as_deref()), (None, None));
        assert_eq!(owner_dto(Some(&owner)).address, "Main Street 1\n12345 Exampletown");
    }

    #[test]
    fn empty_fields_remove_the_details_and_a_missing_name_is_refused() {
        let (done, settings) = applied(plan_owner(&typed("", "\n")));
        assert_eq!((done.as_str(), settings.owner), ("Removed your details", None));

        let SettingsPlan::Refused(message) = plan_owner(&typed("  ", "Main Street 1")) else {
            panic!("expected a refusal")
        };
        assert_eq!(message.text, "Your name must not be blank");
        assert_eq!(owner_dto(None), SettingsOwnerDto::default());
    }
}
