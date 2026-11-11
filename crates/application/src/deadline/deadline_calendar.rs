use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::deadline::{DeadlinePlanner, UpcomingDate};

const LIMIT: usize = 500;

/// The upcoming deadlines and retention dates as an iCalendar file (`.ics`), one all-day event per date, so the
/// user can import them into a calendar application. Pure; writing the file is the UI's.
pub struct DeadlineCalendar;

impl DeadlineCalendar {
    #[must_use]
    pub fn of(catalog: &Catalog, today: NaiveDate) -> String {
        let header = ["BEGIN:VCALENDAR", "VERSION:2.0", "PRODID:-//ragin-dms//deadlines//EN"].map(str::to_owned);
        let events = DeadlinePlanner::upcoming(catalog, today, LIMIT)
            .into_iter()
            .flat_map(|upcoming| event(catalog, &upcoming, today));
        let lines: Vec<String> = header.into_iter().chain(events).chain(["END:VCALENDAR".to_owned()]).collect();
        format!("{}\r\n", lines.join("\r\n"))
    }
}

fn event(catalog: &Catalog, upcoming: &UpcomingDate, today: NaiveDate) -> Vec<String> {
    let document = catalog.documents().get(&upcoming.document_id);
    let name = document
        .and_then(|document| document.metadata.title.clone())
        .or_else(|| document.map(|document| document.path.file_name().to_owned()))
        .unwrap_or_else(|| upcoming.document_id.value().to_owned());
    let what = upcoming.field.as_ref().map_or_else(
        || "Retention ends".to_owned(),
        |key| catalog.settings().field(key).map_or_else(|| key.value().to_owned(), |field| field.label.clone()),
    );
    let day = day(upcoming.date);
    let field = upcoming.field.as_ref().map_or("retention", |key| key.value());
    vec![
        "BEGIN:VEVENT".to_owned(),
        format!("UID:{}-{field}-{day}@ragin-dms", upcoming.document_id),
        format!("DTSTAMP:{}T000000Z", self::day(today)),
        format!("DTSTART;VALUE=DATE:{day}"),
        format!("SUMMARY:{}", escaped(&format!("{what}: {name}"))),
        format!("DESCRIPTION:{}", escaped(document.map_or("", |document| document.path.value()))),
        "END:VEVENT".to_owned(),
    ]
}

/// `yyyyMMdd`; a year beyond four digits gets a `+` sign, in Java's `DateTimeFormatter` and chrono alike.
fn day(date: NaiveDate) -> String {
    date.format("%Y%m%d").to_string()
}

/// RFC 5545 text: backslash, semicolon, comma and line breaks escaped.
fn escaped(text: &str) -> String {
    text.replace('\\', "\\\\").replace(';', "\\;").replace(',', "\\,").replace('\n', "\\n")
}
