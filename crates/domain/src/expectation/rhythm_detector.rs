use chrono::{Datelike, NaiveDate};
use indexmap::IndexMap;

use crate::catalog::Catalog;
use crate::dates;
use crate::document::Document;
use crate::expectation::{ExpectationMode, ExpectationOverride, ExpectedDocument, Rhythm};
use crate::taxonomy::CategoryId;
use crate::text;

/// Learns from the document dates when a sender's next document is due.
///
/// Per sender (ignoring case) and category, with at least [`Self::MINIMUM_DOCUMENTS`] different document dates:
/// the median gap between them decides the [`Rhythm`] when it lies in one of the rhythm's bands and at least 70 %
/// of all gaps lie in that band too. The next document is expected the median gap after the last one, and it is
/// missing once the rhythm's tolerance has passed as well. After two missed periods in a row the sender has
/// probably stopped, and nothing is expected anymore, unless the user chose [`ExpectationMode::Keep`];
/// [`ExpectationMode::Off`] never expects anything.
pub struct RhythmDetector;

/// What the dates of one sender and category show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pattern {
    rhythm: Rhythm,
    median_gap: i64,
    first: NaiveDate,
    last: NaiveDate,
}

impl RhythmDetector {
    pub const MINIMUM_DOCUMENTS: usize = 3;

    /// The next document of every sender with a rhythm that is still expected on `today`, missing or not, by date.
    #[must_use]
    pub fn expected(catalog: &Catalog, today: NaiveDate) -> Vec<ExpectedDocument> {
        let overrides = &catalog.settings().expectations;
        let mut expected: Vec<ExpectedDocument> = groups(catalog.documents().values())
            .into_iter()
            .filter_map(|((_, category_id), group)| {
                let pattern = pattern_of(&group.dates)?;
                let mode = mode_of(overrides, &group.correspondent, category_id.as_ref());
                let still_expected = match mode {
                    Some(ExpectationMode::Off) => false,
                    Some(ExpectationMode::Keep) => true,
                    None => !missed_twice(&pattern, today),
                };
                still_expected.then(|| ExpectedDocument {
                    correspondent: group.correspondent,
                    category_id,
                    expected_on: dates::plus_days(pattern.last, pattern.median_gap),
                    rhythm: pattern.rhythm,
                    first_seen_year: pattern.first.year(),
                })
            })
            .collect();
        expected.sort_by(|first, second| {
            first.expected_on.cmp(&second.expected_on).then_with(|| {
                text::compare(&text::lowercase(&first.correspondent), &text::lowercase(&second.correspondent))
            })
        });
        expected
    }

    /// The expected documents whose tolerance has passed on `today`.
    #[must_use]
    pub fn missing(catalog: &Catalog, today: NaiveDate) -> Vec<ExpectedDocument> {
        Self::expected(catalog, today)
            .into_iter()
            .filter(|expected| today > dates::plus_days(expected.expected_on, expected.rhythm.tolerance_days()))
            .collect()
    }
}

struct Group {
    correspondent: String,
    dates: Vec<NaiveDate>,
}

/// The dated documents per sender (lower case) and category; the name is the spelling of the latest document.
fn groups<'d>(documents: impl Iterator<Item = &'d Document>) -> IndexMap<(String, Option<CategoryId>), Group> {
    let mut groups: IndexMap<(String, Option<CategoryId>), Group> = IndexMap::new();
    for document in documents {
        let (Some(correspondent), Some(date)) =
            (document.metadata.correspondent.as_deref(), document.metadata.document_date)
        else {
            continue;
        };
        let key = (text::lowercase(text::trim(correspondent)), document.category_id.clone());
        let group =
            groups.entry(key).or_insert_with(|| Group { correspondent: correspondent.to_owned(), dates: Vec::new() });
        if group.dates.iter().all(|known| *known < date) {
            correspondent.clone_into(&mut group.correspondent);
        }
        group.dates.push(date);
    }
    groups
}

fn pattern_of(dates: &[NaiveDate]) -> Option<Pattern> {
    let mut dates = dates.to_vec();
    dates.sort_unstable();
    dates.dedup();
    if dates.len() < RhythmDetector::MINIMUM_DOCUMENTS {
        return None;
    }
    let gaps: Vec<i64> = dates.windows(2).map(|pair| (pair[1] - pair[0]).num_days()).collect();
    let median_gap = median(&gaps);
    let rhythm = Rhythm::of_gap(median_gap)?;
    let in_band = gaps.iter().filter(|gap| rhythm.contains(**gap)).count();
    // At least 70 % of the gaps, in whole numbers.
    if in_band * 10 < gaps.len() * 7 {
        return None;
    }
    Some(Pattern { rhythm, median_gap, first: dates[0], last: dates[dates.len() - 1] })
}

/// The middle gap; of an even number the mean of the two middle ones, rounded down.
fn median(gaps: &[i64]) -> i64 {
    let mut sorted = gaps.to_vec();
    sorted.sort_unstable();
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) { (sorted[middle - 1] + sorted[middle]).div_euclid(2) } else { sorted[middle] }
}

/// `true` once the second period after the last document has passed, tolerance included.
fn missed_twice(pattern: &Pattern, today: NaiveDate) -> bool {
    let second = dates::plus_days(pattern.last, pattern.median_gap.saturating_mul(2));
    today > dates::plus_days(second, pattern.rhythm.tolerance_days())
}

fn mode_of(
    overrides: &[ExpectationOverride],
    correspondent: &str,
    category_id: Option<&CategoryId>,
) -> Option<ExpectationMode> {
    let wanted = (text::lowercase(text::trim(correspondent)), category_id.cloned());
    overrides.iter().find(|candidate| candidate.key() == wanted).map(|candidate| candidate.mode)
}
