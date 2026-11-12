use std::sync::Arc;

use chrono::DateTime;
use dms_application::common::OperationOutcome;
use dms_application::mail::{EmailSource, ImportEmails, ImportEmailsOutcome, ImportedEmail};
use dms_application::port::mail::{MailAttachment, MailMessage};

use crate::support::TestLibrary;
use crate::support::fake::mail::FakeMailConverter;
use crate::support::fake::pdf::FakePdfEditor;
use crate::support::test_data::{date, path};

struct MailKit {
    library: TestLibrary,
    mail: Arc<FakeMailConverter>,
}

impl MailKit {
    fn new() -> Self {
        let library = TestLibrary::new();
        library.open();
        Self { mail: Arc::new(FakeMailConverter::new(library.folders.clone())), library }
    }

    fn import(&self, sources: &[EmailSource], keep_original: bool) -> Vec<ImportedEmail> {
        let library = &self.library;
        let use_case = ImportEmails::new(
            library.store.clone(),
            self.mail.clone(),
            Arc::new(FakePdfEditor::new(library.folders.clone())),
            library.replicator.clone(),
            library.names.clone(),
            Arc::new(library.trash_document()),
            library.activity.clone(),
            library.clock.clone(),
            library.ids.clone(),
        );
        match use_case.run(sources, keep_original) {
            ImportEmailsOutcome::Imported(imported) => imported,
            ImportEmailsOutcome::LibraryNotOpen => panic!("expected Imported"),
        }
    }
}

fn invoice_mail() -> MailMessage {
    MailMessage {
        from_name: Some("Stadtwerke Nordhafen".to_owned()),
        from_address: Some("rechnung@stadtwerke.example".to_owned()),
        to: vec!["Erika Beispiel <erika@example.org>".to_owned()],
        subject: Some("Ihre Rechnung für Oktober".to_owned()),
        sent_at: Some(DateTime::parse_from_rfc3339("2026-11-03T23:30:00+01:00").expect("date")),
        body: "Guten Tag,\n\nanbei Ihre Rechnung.".to_owned(),
        attachments: vec![
            MailAttachment { name: "Rechnung.pdf".to_owned(), size: 10 },
            MailAttachment { name: "AGB.pdf".to_owned(), size: 10 },
        ],
    }
}

#[test]
fn a_mail_in_the_inbox_becomes_a_pdf_and_linked_attachments_and_its_file_goes_to_the_trash() {
    let kit = MailKit::new();
    let eml = kit.library.add("Inbox/rechnung.eml", "raw mail");
    kit.library.folders.put(&kit.library.primary, "Inbox/Rechnung.pdf", "an older file of that name");
    kit.mail.set("rechnung.eml", invoice_mail());

    let imported = kit.import(&[EmailSource::Document(eml.id.clone())], false);

    let [email] = imported.as_slice() else { panic!("one mail") };
    assert!(matches!(email.outcome, OperationOutcome::Done { .. }), "{:?}", email.outcome);
    let catalog = kit.library.catalog();
    let mail = catalog.documents().get(email.mail.as_ref().expect("a mail document")).expect("catalogued");
    assert_eq!(mail.path.value(), "Inbox/2026-11-03 Ihre Rechnung für Oktober.pdf");
    assert_eq!(mail.metadata.title.as_deref(), Some("Ihre Rechnung für Oktober"));
    assert_eq!(mail.metadata.correspondent.as_deref(), Some("Stadtwerke Nordhafen"));
    assert_eq!(mail.metadata.document_date, Some(date(2026, 11, 3)));
    let attachments: Vec<&str> =
        email.attachments.iter().map(|id| catalog.documents().get(id).expect("catalogued").path.value()).collect();
    assert_eq!(attachments, vec!["Inbox/Rechnung (2).pdf", "Inbox/AGB.pdf"]);
    let links = catalog.links_of(&mail.id);
    assert_eq!(links.len(), 2);
    assert!(links.iter().all(|link| link.label() == Some("attachment")));
    for folder in kit.library.all_folders() {
        let pdf = kit.library.folders.content(&folder, mail.path.value()).expect("the mail PDF in every folder");
        assert!(pdf.contains("From: Stadtwerke Nordhafen <rechnung@stadtwerke.example>"), "{pdf}");
        assert!(pdf.contains("Date: 2026-11-03 23:30 +01:00"), "{pdf}");
        assert!(kit.library.folders.content(&folder, "Inbox/AGB.pdf").is_some());
    }
    assert!(!catalog.documents().contains_key(&eml.id), "the mail file left the catalog");
    assert_eq!(kit.library.folders.trashed().len(), 3, "the mail file went to the trash in every folder");
}

#[test]
fn the_original_stays_when_it_should_be_kept_and_a_mail_without_sender_name_takes_the_domain() {
    let kit = MailKit::new();
    let eml = kit.library.add("Inbox/note.eml", "raw mail");
    kit.mail.set("note.eml", MailMessage { from_name: None, subject: None, attachments: Vec::new(), ..invoice_mail() });

    let imported = kit.import(&[EmailSource::Document(eml.id.clone())], true);

    let catalog = kit.library.catalog();
    let mail = catalog.documents().get(imported[0].mail.as_ref().expect("mail")).expect("catalogued");
    assert_eq!(mail.path.value(), "Inbox/2026-11-03 Email.pdf");
    assert_eq!(mail.metadata.correspondent.as_deref(), Some("stadtwerke.example"));
    assert!(catalog.documents().contains_key(&eml.id));
    assert_eq!(kit.library.folders.trashed().len(), 0);
}

#[test]
fn a_dropped_mail_goes_into_the_chosen_folder_and_other_files_are_refused() {
    let kit = MailKit::new();
    let dropped = kit.library.folders.external("/outside/rechnung.eml", "raw mail");
    let other = kit.library.folders.external("/outside/photo.jpg", "jpeg");
    kit.mail.set("rechnung.eml", invoice_mail());

    let imported = kit.import(
        &[
            EmailSource::External { file: dropped, folder: path("Bills") },
            EmailSource::External { file: other, folder: path("Bills") },
        ],
        false,
    );

    assert!(matches!(imported[0].outcome, OperationOutcome::Done { .. }), "{:?}", imported[0].outcome);
    assert!(matches!(imported[1].outcome, OperationOutcome::InvalidInput(_)), "{:?}", imported[1].outcome);
    let paths = kit.library.folders.paths(&kit.library.primary);
    assert!(paths.contains("Bills/2026-11-03 Ihre Rechnung für Oktober.pdf"), "{paths:?}");
    assert!(paths.contains("Bills/Rechnung.pdf"), "{paths:?}");
}

#[test]
fn an_unreadable_mail_changes_nothing() {
    let kit = MailKit::new();
    let eml = kit.library.add("Inbox/broken.eml", "not a mail");
    let before = kit.library.folders.paths(&kit.library.primary);

    let imported = kit.import(&[EmailSource::Document(eml.id)], false);

    assert!(matches!(imported[0].outcome, OperationOutcome::StorageFailed(_)), "{:?}", imported[0].outcome);
    assert_eq!(kit.library.folders.paths(&kit.library.primary), before);
}
