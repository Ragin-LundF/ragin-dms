//! Words the document area computes: relative days (Kotlin `DateTexts.relative`), typed dates and names
//! (`DateInput`, `NameInput`) and what the AI answered (`AiMessages`).

use chrono::NaiveDate;
use dms_application::assistant::AnswerOutcome;
use dms_application::classification::SuggestOutcome;
use dms_application::port::ai::ClassificationFailure;
use dms_domain::library::FileNamePolicy;
use dms_domain::{parse_iso_date, text};

use crate::messages;
use crate::session::UiMessage;

/// The one message of every date field.
pub const DATE_PROBLEM: &str = "Write the date as YYYY-MM-DD, such as 2030-12-31";

const DAYS_PER_WEEK: i64 = 7;
const DAYS_PER_MONTH: i64 = 30;
const DAYS_PER_YEAR: i64 = 365;

/// The date in `raw` (`YYYY-MM-DD`), or `None`; surrounding spaces do not count.
pub fn parse_date(raw: &str) -> Option<NaiveDate> {
    parse_iso_date(text::trim(raw))
}

/// Why `raw` cannot be a file or folder name, or `None` if it can; `raw` is trimmed first.
pub fn name_problem(raw: &str) -> Option<&'static str> {
    FileNamePolicy::check(text::trim(raw)).map(messages::name_problem)
}

/// `date` relative to `today`: "today", "yesterday", "tomorrow", "N days ago", "in N weeks", …
pub fn relative_day(date: NaiveDate, today: NaiveDate) -> String {
    let days = (date - today).num_days();
    match days {
        0 => "today".to_owned(),
        -1 => "yesterday".to_owned(),
        1 => "tomorrow".to_owned(),
        _ if days < 0 => format!("{} ago", amount_of(-days)),
        _ => format!("in {}", amount_of(days)),
    }
}

fn amount_of(days: i64) -> String {
    if days < DAYS_PER_WEEK * 2 {
        plural(days, "day")
    } else if days < DAYS_PER_MONTH * 2 {
        plural(days / DAYS_PER_WEEK, "week")
    } else if days < DAYS_PER_YEAR {
        plural(days / DAYS_PER_MONTH, "month")
    } else {
        plural(days / DAYS_PER_YEAR, "year")
    }
}

fn plural(count: i64, unit: &str) -> String {
    if count == 1 { format!("1 {unit}") } else { format!("{count} {unit}s") }
}

/// The message for a suggestion run on the file `name`.
pub fn suggest_message(outcome: &SuggestOutcome, name: &str) -> UiMessage {
    match outcome {
        SuggestOutcome::Suggested(_) => UiMessage::info(format!("The AI suggested a classification for {name}")),
        SuggestOutcome::LibraryNotOpen => UiMessage::error("No library is open"),
        SuggestOutcome::NotFound => UiMessage::error(format!("{name} is no longer in the library")),
        SuggestOutcome::NotReadable { detail } => UiMessage::error(not_readable(name, detail.as_deref())),
        SuggestOutcome::Failed { reason, .. } => UiMessage::error(failure(*reason)),
    }
}

/// Why a summary or an answer about the file `name` is missing; `None` when there is one.
pub fn answer_problem(outcome: &AnswerOutcome, name: &str) -> Option<String> {
    match outcome {
        AnswerOutcome::Answered { .. } => None,
        AnswerOutcome::LibraryNotOpen => Some("No library is open".to_owned()),
        AnswerOutcome::NotFound => Some(format!("{name} is no longer in the library")),
        AnswerOutcome::NotReadable { detail } => Some(not_readable(name, detail.as_deref())),
        AnswerOutcome::Failed { reason, .. } => Some(failure(*reason).to_owned()),
    }
}

pub fn failure(reason: ClassificationFailure) -> &'static str {
    match reason {
        ClassificationFailure::ProviderUnavailable => {
            "The AI does not answer. Start LM Studio's server, or for Anthropic set ANTHROPIC_API_KEY; check Settings › AI"
        }
        ClassificationFailure::ModelCannotReadImages => {
            "The model cannot read images. Load a vision model, such as Gemma 3 or Qwen2.5-VL"
        }
        ClassificationFailure::InvalidResponse => "The AI's answer could not be understood; try again",
        ClassificationFailure::Timeout => "The AI took too long; try again, or raise the timeout in Settings › AI",
    }
}

fn not_readable(name: &str, detail: Option<&str>) -> String {
    match detail {
        Some(detail) => format!("{name} could not be read for the AI: {detail}"),
        None => format!("{name} could not be read for the AI"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(text: &str) -> NaiveDate {
        parse_iso_date(text).expect("test date")
    }

    #[test]
    fn relative_day_uses_words_near_today_and_coarser_units_further_away() {
        let today = day("2026-03-10");
        let cases = [
            ("2026-03-10", "today"),
            ("2026-03-09", "yesterday"),
            ("2026-03-11", "tomorrow"),
            ("2026-03-23", "in 13 days"),
            ("2026-03-24", "in 2 weeks"),
            ("2026-02-24", "2 weeks ago"),
            ("2026-05-09", "in 2 months"),
            ("2027-03-10", "in 1 year"),
            ("2026-03-03", "7 days ago"),
        ];
        for (date, expected) in cases {
            assert_eq!(relative_day(day(date), today), expected, "{date}");
        }
    }

    #[test]
    fn parse_date_ignores_surrounding_spaces_and_refuses_other_forms() {
        assert_eq!(parse_date(" 2030-12-31 "), Some(day("2030-12-31")));
        assert_eq!(parse_date("31.12.2030"), None);
        assert_eq!(parse_date("2030-02-30"), None);
    }

    #[test]
    fn name_problem_trims_before_checking() {
        assert_eq!(name_problem("  letter.pdf "), None);
        assert_eq!(name_problem("   "), Some("The name is empty"));
        assert_eq!(name_problem("a:b.pdf"), Some("The name contains one of < > : \" / \\ | ? *"));
    }

    #[test]
    fn not_readable_messages_add_the_detail_when_there_is_one() {
        let outcome = AnswerOutcome::NotReadable { detail: Some("The PDF has no pages".to_owned()) };
        assert_eq!(
            answer_problem(&outcome, "scan.pdf").as_deref(),
            Some("scan.pdf could not be read for the AI: The PDF has no pages")
        );
        let message = suggest_message(&SuggestOutcome::NotReadable { detail: None }, "scan.pdf");
        assert_eq!(message, UiMessage::error("scan.pdf could not be read for the AI"));
    }
}
