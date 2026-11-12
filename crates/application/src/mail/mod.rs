//! Importing emails: a mail file becomes a PDF of the mail plus one document per attachment, linked.

mod import_emails;
mod mail_document;
pub mod model;

pub use import_emails::ImportEmails;
pub use mail_document::{is_mail_file, mail_document, mail_file_name};
pub use model::{EmailSource, ImportEmailsOutcome, ImportedEmail};
