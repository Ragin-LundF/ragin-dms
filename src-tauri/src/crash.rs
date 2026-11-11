//! The "Ragin-DMS stopped working" dialog (Kotlin `AppErrorHandler` + `ErrorTexts`): a panic anywhere shows what
//! happened in words a person can act on, then the app closes instead of hanging or vanishing.

use std::panic::PanicHookInfo;

pub const TITLE: &str = "Ragin-DMS stopped working";

/// Installs the hook once at start; the default hook still prints the panic for the terminal.
pub fn install() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default(info);
        // A message dialog from the panicking thread; rfd runs it on the main thread where the platform needs it.
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title(TITLE)
            .set_description(message_of(info))
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        std::process::exit(1);
    }));
}

/// "Something unexpected went wrong (Panic: detail). …", as Kotlin wrote it for an exception and its message.
pub fn message_of(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let detail =
        payload.downcast_ref::<&str>().map(ToString::to_string).or_else(|| payload.downcast_ref::<String>().cloned());
    message(detail.as_deref())
}

fn message(detail: Option<&str>) -> String {
    let detail = detail.filter(|text| !text.trim().is_empty()).map(|text| format!(": {text}")).unwrap_or_default();
    format!("Something unexpected went wrong (Panic{detail}). The window closes now; open Ragin-DMS again to continue.")
}

#[cfg(test)]
mod tests {
    use super::message;

    #[test]
    fn names_the_failure_like_the_kotlin_dialog() {
        assert_eq!(
            message(Some("invariant: x")),
            "Something unexpected went wrong (Panic: invariant: x). The window closes now; open Ragin-DMS again to continue."
        );
        assert_eq!(
            message(Some("  ")),
            "Something unexpected went wrong (Panic). The window closes now; open Ragin-DMS again to continue."
        );
    }
}
