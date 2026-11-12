//! Reading email files (`.eml`) and writing their attachments.

mod mail_converter;
pub mod model;

pub use mail_converter::MailConverter;
pub use model::{MailAttachment, MailMessage, MailSource};
