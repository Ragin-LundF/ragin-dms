use dms_application::common::OperationOutcome;
use dms_application::emergency::{EmergencyEdit, EmergencyOutcome, OverviewTarget};
use dms_domain::emergency::{EmergencyChanges, EmergencyEntry, EmergencyGroup};

use super::kit::EmergencyKit;
use crate::support::test_data::{date, done, path};

fn contact(name: &str) -> EmergencyEntry {
    EmergencyEntry::Contact {
        name: name.to_owned(),
        role: Some("Tax adviser".to_owned()),
        phone: Some("040 123456".to_owned()),
        email: None,
        note: None,
    }
}

#[test]
fn items_are_added_once_switched_off_and_removed() {
    let kit = EmergencyKit::new();
    let policy = kit.add("insurance/policy.pdf", "life insurance");

    kit.add_item(EmergencyGroup::Insurance, EmergencyEntry::Document(policy.id.clone()));
    kit.add_item(EmergencyGroup::Insurance, EmergencyEntry::Document(policy.id.clone()));
    kit.add_item(EmergencyGroup::Contacts, contact("Kanzlei Berg"));
    let items = kit.folder().items;
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|item| item.included));

    done(kit.edit().run(&EmergencyEdit::SetIncluded { id: items[0].id.clone(), included: false }));
    assert!(!kit.folder().items[0].included);

    done(kit.edit().run(&EmergencyEdit::Remove(items[1].id.clone())));
    assert_eq!(kit.folder().items.len(), 1);
    assert_eq!(kit.edit().run(&EmergencyEdit::Remove(items[1].id.clone())), OperationOutcome::NotFound);
}

#[test]
fn a_contact_without_a_name_is_refused() {
    let kit = EmergencyKit::new();

    let outcome = kit.edit().run(&EmergencyEdit::Add { group: EmergencyGroup::Contacts, entry: contact(" ") });

    assert!(matches!(outcome, OperationOutcome::InvalidInput(_)), "{outcome:?}");
}

#[test]
fn after_a_refresh_an_edited_document_shows_as_changed() {
    let kit = EmergencyKit::new();
    let policy = kit.add("insurance/policy.pdf", "life insurance");
    kit.add_item(EmergencyGroup::Insurance, EmergencyEntry::Document(policy.id.clone()));

    done(kit.mark_refreshed().run());
    let folder = kit.folder();
    assert_eq!(folder.last_refreshed, Some(date(2024, 3, 1)));
    assert_eq!(EmergencyChanges::of(&folder, &kit.catalog()).len(), 0);

    let edited = crate::support::fake::storage::FakeFolders::hash_of("life insurance, new terms");
    done(kit.store.commit(|catalog| Ok(catalog.update_content(&policy.id, edited.clone(), 25))));

    assert_eq!(EmergencyChanges::of(&kit.folder(), &kit.catalog()).len(), 1);
}

#[test]
fn the_overview_saved_in_the_library_is_catalogued_in_every_folder() {
    let kit = EmergencyKit::new();
    let policy = kit.add("insurance/policy.pdf", "life insurance");
    kit.add_item(EmergencyGroup::Insurance, EmergencyEntry::Document(policy.id));
    kit.add_item(EmergencyGroup::Contacts, contact("Kanzlei Berg"));

    let outcome = kit.create_overview().run(&OverviewTarget::Library { folder: path("personal") });

    let EmergencyOutcome::Saved { location, document_id: Some(id), .. } = outcome else {
        panic!("expected Saved, got {outcome:?}")
    };
    assert_eq!(location, "personal/Emergency overview 2024-03-01.pdf");
    for folder in kit.all_folders() {
        let content = kit.folders.content(&folder, &location).expect("in every folder");
        assert!(content.contains("Contacts\tKanzlei Berg · Tax adviser · 040 123456"), "{content}");
        assert!(content.contains("Insurance\tpolicy.pdf"), "{content}");
        assert!(content.find("Contacts\t") < content.find("Insurance\t"), "groups in their fixed order: {content}");
    }
    let catalog = kit.catalog();
    let overview = catalog.documents().get(&id).expect("catalogued");
    assert_eq!(overview.metadata.title.as_deref(), Some("Emergency overview"));
}

#[test]
fn the_overview_outside_the_library_is_refused_inside_a_library_folder() {
    let kit = EmergencyKit::new();
    kit.add_item(EmergencyGroup::Contacts, contact("Kanzlei Berg"));

    let inside = kit
        .create_overview()
        .run(&OverviewTarget::Outside { directory: "/library/sync-a/sub".to_owned(), name: "overview.pdf".to_owned() });
    let outside = kit
        .create_overview()
        .run(&OverviewTarget::Outside { directory: "/home/print".to_owned(), name: "overview.pdf".to_owned() });

    assert_eq!(inside, EmergencyOutcome::InsideLibrary);
    assert!(matches!(outside, EmergencyOutcome::Saved { document_id: None, .. }), "{outside:?}");
    let files = kit.exports.files.get();
    assert!(String::from_utf8_lossy(&files["/home/print/overview.pdf"]).contains("Kanzlei Berg"));
}

#[test]
fn the_encrypted_zip_holds_the_overview_and_the_included_documents_and_their_contracts() {
    let kit = EmergencyKit::new();
    let policy = kit.add("insurance/policy.pdf", "life insurance");
    let left_out = kit.add("personal/passport.pdf", "passport copy");
    let rent = kit.add("home/rent.pdf", "rental contract");
    let contract = dms_domain::contract::Contract {
        document_ids: [rent.id.clone()].into_iter().collect(),
        ..dms_domain::contract::Contract::new(
            dms_domain::contract::ContractId::new("c-1").expect("id"),
            "Rent",
            dms_domain::contract::ContractKind::Other,
        )
        .expect("contract")
    };
    done(kit.store.commit(|catalog| catalog.save_contract(contract.clone())));
    kit.add_item(EmergencyGroup::Insurance, EmergencyEntry::Document(policy.id));
    kit.add_item(EmergencyGroup::Personal, EmergencyEntry::Document(left_out.id));
    kit.add_item(EmergencyGroup::Contracts, EmergencyEntry::Contract(contract.id));
    let passport = kit.folder().items[1].id.clone();
    done(kit.edit().run(&EmergencyEdit::SetIncluded { id: passport, included: false }));

    let outcome = kit.export().run("/home/usb", "Emergency folder", "correct horse");

    assert!(matches!(outcome, EmergencyOutcome::Saved { documents: 2, .. }), "{outcome:?}");
    let written = kit.exports.encrypted.get();
    let (bundle, extra, target, password) = &written[0];
    let mut sources: Vec<&str> = bundle.entries.iter().map(|entry| entry.source.value()).collect();
    sources.sort_unstable();
    assert_eq!(sources, vec!["home/rent.pdf", "insurance/policy.pdf"]);
    assert_eq!(bundle.index, None);
    assert_eq!(extra[0].0, "Emergency overview.pdf");
    assert_eq!(target.name, "Emergency folder");
    assert_eq!(password, "correct horse");
}

#[test]
fn the_zip_needs_a_password_of_eight_characters_a_place_outside_and_an_included_item() {
    let kit = EmergencyKit::new();

    assert_eq!(kit.export().run("/home/usb", "Emergency", "longenough"), EmergencyOutcome::NothingIncluded);
    kit.add_item(EmergencyGroup::Contacts, contact("Kanzlei Berg"));
    assert!(matches!(
        kit.export().run("/home/usb", "Emergency", "short"),
        EmergencyOutcome::Refused(OperationOutcome::InvalidInput(_))
    ));
    assert_eq!(kit.export().run("/library/primary", "Emergency", "longenough"), EmergencyOutcome::InsideLibrary);
    assert_eq!(kit.exports.encrypted.get().len(), 0);
}
