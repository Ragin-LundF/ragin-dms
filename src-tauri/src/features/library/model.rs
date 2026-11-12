//! The library's view parameters as domain types (Kotlin `LibraryViewStateModel`, `FacetFilter`, `GroupKey`, ...),
//! parsed from the DTOs at the command boundary so the presenters work on checked values.

use chrono::{DateTime, Datelike, NaiveDate, TimeDelta, TimeZone, Utc};
use chrono_tz::Tz;
use dms_domain::bill::BillStatus;
use dms_domain::document::DocumentKind;
use dms_domain::library::RelativePath;
use dms_domain::query::{DateRange, FieldCondition, SavedSearchId};
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::{CategoryId, TagName};

/// A view of the sidebar (Kotlin `LibrarySelection` as the library stores it): "All documents" is the root folder.
/// A category, tag or folder below the root, as the dashboard opens them, arrive as filters of "All documents".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Folder(RelativePath),
    Inbox,
    NeedsReview,
    Recent,
    Folders,
    SmartFolder(SavedSearchId),
}

/// One active facet filter; at most one per facet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacetFilter {
    /// The category with its subcategories; `None` for the documents without a category.
    Category(Option<CategoryId>),
    Tag(TagName),
    Year(YearFilter),
    /// The documents from this correspondent, ignoring case.
    Correspondent(String),
    Type(DocumentKind),
    /// The bills that are open, overdue or paid.
    Payment(BillStatus),
    /// The folder with everything below it.
    Folder(RelativePath),
    /// The documents assigned to a tax year, or to one area of it.
    Tax(TaxFacet),
    /// Document dates in this range, as "Turn into filters" makes it from a sentence.
    Dates(DateRange),
    /// A condition on a custom field, as "Turn into filters" makes it from a sentence.
    Field(FieldCondition),
}

/// A tax year, or one of its areas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaxFacet {
    pub year: i32,
    pub area: Option<TaxArea>,
}

impl TaxFacet {
    pub fn label(self) -> String {
        match self.area {
            Some(area) => format!("{} · {}", self.year, area.label()),
            None => self.year.to_string(),
        }
    }
}

/// Documents of one `year`, or of that year and every earlier one (Kotlin `YearFilterModel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearFilter {
    pub year: i32,
    pub and_earlier: bool,
}

impl YearFilter {
    pub fn label(self) -> String {
        if self.and_earlier { format!("Until {}", self.year) } else { self.year.to_string() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facet {
    Category,
    Tag,
    Year,
    Correspondent,
    Type,
    Payment,
    Folder,
    Tax,
}

impl Facet {
    pub const ALL: [Facet; 8] = [
        Self::Category,
        Self::Tag,
        Self::Year,
        Self::Correspondent,
        Self::Type,
        Self::Payment,
        Self::Folder,
        Self::Tax,
    ];

    /// The start of the filter's token in the search field, as in `from: Sample Utility`.
    pub fn key(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::Tag => "tag",
            Self::Year => "year",
            Self::Correspondent => "from",
            Self::Type => "type",
            Self::Payment => "payment",
            Self::Folder => "folder",
            Self::Tax => "tax",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKey {
    Month,
    Category,
    Sender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Category,
    Correspondent,
    Date,
    Amount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub column: SortColumn,
    pub descending: bool,
}

/// What the user chose on the library screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewParams {
    pub view: View,
    pub filters: Vec<FacetFilter>,
    pub query: String,
    pub sort: Option<Sort>,
    pub group_by: GroupKey,
    /// Whether the search reads the documents' text too; a saved view then keeps the text as its content term.
    pub content_search: bool,
}

/// Now, in the zone the library shows dates in.
#[derive(Debug, Clone, Copy)]
pub struct Moment {
    pub now: DateTime<Utc>,
    pub zone: Tz,
}

impl Moment {
    pub fn today(&self) -> NaiveDate {
        self.now.with_timezone(&self.zone).date_naive()
    }

    /// Where "Added this month" begins: the first day of this calendar month, at its first moment in the zone.
    pub fn start_of_month(&self) -> DateTime<Utc> {
        let first = self.today().with_day(1).expect("invariant: every month has a first day");
        let midnight = first.and_time(chrono::NaiveTime::MIN);
        // A clock change at midnight skips it; like Java's atStartOfDay, the day then starts after the gap.
        (0..3)
            .find_map(|hours| self.zone.from_local_datetime(&(midnight + TimeDelta::hours(hours))).earliest())
            .map_or(self.now, |start| start.with_timezone(&Utc))
    }

    pub fn year_of(&self, instant: DateTime<Utc>) -> i32 {
        instant.with_timezone(&self.zone).year()
    }
}
