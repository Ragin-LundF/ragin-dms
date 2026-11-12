//! What the library screen sends: its view parameters, and their parsing into checked domain values.

use chrono::NaiveDate;
use dms_domain::bill::BillStatus;
use dms_domain::document::{DocumentId, DocumentKind};
use dms_domain::field::{FieldKey, FieldType, FieldValue, FieldValues};
use dms_domain::library::RelativePath;
use dms_domain::query::{DateRange, FieldComparison, FieldCondition, SavedSearchId};
use dms_domain::taxonomy::{CategoryId, TagName};
use dms_domain::{iso_date, parse_iso_date};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::dto::IpcError;
use crate::features::library::model::{
    FacetFilter, GroupKey, Sort, SortColumn, TaxFacet, View, ViewParams, YearFilter,
};
use crate::features::settings::dto::FieldTypeDto;
use crate::features::tax::dto::TaxAreaDto;
use crate::features::tax::view::amounts::{area_dto, area_of};

/// A view of the sidebar: `folder` with the root path is "All documents".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryViewDto {
    Folder { path: String },
    Inbox,
    NeedsReview,
    Recent,
    Folders,
    SmartFolder { id: String },
}

/// One active facet filter; `category` with `categoryId` null is "Without a category".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryFilterDto {
    #[serde(rename_all = "camelCase")]
    Category {
        category_id: Option<String>,
    },
    Tag {
        tag: String,
    },
    #[serde(rename_all = "camelCase")]
    Year {
        year: i32,
        and_earlier: bool,
    },
    Correspondent {
        name: String,
    },
    #[serde(rename_all = "camelCase")]
    Type {
        document_kind: LibraryKindDto,
    },
    Payment {
        status: LibraryPaymentDto,
    },
    Folder {
        path: String,
    },
    /// A tax year, or with `area` one of its areas.
    Tax {
        year: i32,
        area: Option<TaxAreaDto>,
    },
    /// Document dates from `from` to `to` (`YYYY-MM-DD`, both included); a missing end has no limit.
    Dates {
        from: Option<String>,
        to: Option<String>,
    },
    /// A condition on the custom field `key`; `value` as the field shows it, of `value_type`, empty for
    /// `present` and `absent`.
    #[serde(rename_all = "camelCase")]
    Field {
        key: String,
        comparison: LibraryComparisonDto,
        value: Option<String>,
        value_type: Option<FieldTypeDto>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LibraryPaymentDto {
    Open,
    Overdue,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LibraryComparisonDto {
    Present,
    Absent,
    Equals,
    Contains,
    LessThan,
    GreaterThan,
    WithinDays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LibraryKindDto {
    Pdf,
    Image,
    Office,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LibraryGroupKeyDto {
    Month,
    Category,
    Sender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LibrarySortColumnDto {
    Name,
    Category,
    Correspondent,
    Date,
    Amount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySortDto {
    pub column: LibrarySortColumnDto,
    pub descending: bool,
}

/// What the user chose on the library screen (Kotlin `LibraryViewStateModel` without the pure UI parts).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryParamsDto {
    pub view: LibraryViewDto,
    pub filters: Vec<LibraryFilterDto>,
    pub query: String,
    pub sort: Option<LibrarySortDto>,
    pub group_by: LibraryGroupKeyDto,
    /// Whether the search reads the documents' text too.
    pub content_search: bool,
}

impl LibraryParamsDto {
    /// # Errors
    /// `invalid_input` for an id, path, tag or year that cannot be one.
    pub fn parse(&self) -> Result<ViewParams, IpcError> {
        Ok(ViewParams {
            view: self.view.parse()?,
            filters: self.filters.iter().map(LibraryFilterDto::parse).collect::<Result<_, _>>()?,
            query: self.query.clone(),
            sort: self.sort.map(|sort| Sort { column: sort.column.into(), descending: sort.descending }),
            group_by: self.group_by.into(),
            content_search: self.content_search,
        })
    }
}

impl LibraryViewDto {
    fn parse(&self) -> Result<View, IpcError> {
        Ok(match self {
            Self::Folder { path } => View::Folder(parse_path(path)?),
            Self::Inbox => View::Inbox,
            Self::NeedsReview => View::NeedsReview,
            Self::Recent => View::Recent,
            Self::Folders => View::Folders,
            Self::SmartFolder { id } => View::SmartFolder(parse_saved_search_id(id)?),
        })
    }
}

impl LibraryFilterDto {
    pub(crate) fn parse(&self) -> Result<FacetFilter, IpcError> {
        Ok(match self {
            Self::Category { category_id } => {
                FacetFilter::Category(category_id.as_deref().map(parse_category_id).transpose()?)
            }
            Self::Tag { tag } => FacetFilter::Tag(parse_tag(tag)?),
            Self::Year { year, and_earlier } => {
                DateRange::year(*year).map_err(invalid)?;
                FacetFilter::Year(YearFilter { year: *year, and_earlier: *and_earlier })
            }
            Self::Correspondent { name } => FacetFilter::Correspondent(name.clone()),
            Self::Type { document_kind } => FacetFilter::Type((*document_kind).into()),
            Self::Payment { status } => FacetFilter::Payment(match status {
                LibraryPaymentDto::Open => BillStatus::Open,
                LibraryPaymentDto::Overdue => BillStatus::Overdue,
                LibraryPaymentDto::Paid => BillStatus::Paid,
            }),
            Self::Folder { path } => FacetFilter::Folder(parse_path(path)?),
            Self::Tax { year, area } => {
                DateRange::year(*year).map_err(invalid)?;
                FacetFilter::Tax(TaxFacet { year: *year, area: area.map(area_of) })
            }
            Self::Dates { from, to } => FacetFilter::Dates(
                DateRange::new(parse_date(from.as_deref())?, parse_date(to.as_deref())?).map_err(invalid)?,
            ),
            Self::Field { key, comparison, value, value_type } => {
                FacetFilter::Field(field_condition(key, *comparison, value.as_deref(), *value_type)?)
            }
        })
    }

    pub fn of(filter: &FacetFilter) -> Self {
        match filter {
            FacetFilter::Category(id) => Self::Category { category_id: id.as_ref().map(|id| id.value().to_owned()) },
            FacetFilter::Tag(tag) => Self::Tag { tag: tag.value().to_owned() },
            FacetFilter::Year(year) => Self::Year { year: year.year, and_earlier: year.and_earlier },
            FacetFilter::Correspondent(name) => Self::Correspondent { name: name.clone() },
            FacetFilter::Type(kind) => Self::Type { document_kind: (*kind).into() },
            FacetFilter::Payment(status) => Self::Payment {
                status: match status {
                    BillStatus::Open => LibraryPaymentDto::Open,
                    BillStatus::Overdue => LibraryPaymentDto::Overdue,
                    BillStatus::Paid => LibraryPaymentDto::Paid,
                },
            },
            FacetFilter::Folder(path) => Self::Folder { path: path.value().to_owned() },
            FacetFilter::Tax(tax) => Self::Tax { year: tax.year, area: tax.area.map(area_dto) },
            FacetFilter::Dates(range) => Self::Dates { from: range.from.map(iso_date), to: range.to.map(iso_date) },
            FacetFilter::Field(condition) => Self::Field {
                key: condition.key.value().to_owned(),
                comparison: comparison_dto(condition.comparison),
                value: condition.value.as_ref().map(FieldValues::format),
                value_type: condition.value.as_ref().map(value_type_of),
            },
        }
    }
}

fn parse_date(raw: Option<&str>) -> Result<Option<NaiveDate>, IpcError> {
    raw.map(|raw| parse_iso_date(raw).ok_or_else(|| IpcError::invalid(format!("Not a date: {raw}")))).transpose()
}

fn field_condition(
    key: &str,
    comparison: LibraryComparisonDto,
    value: Option<&str>,
    value_type: Option<FieldTypeDto>,
) -> Result<FieldCondition, IpcError> {
    let value = match (value, value_type) {
        (Some(raw), Some(value_type)) => Some(
            FieldValues::parse(field_type_of(value_type), raw, None)
                .ok_or_else(|| IpcError::invalid(format!("Not a value of this field: {raw}")))?,
        ),
        _ => None,
    };
    FieldCondition::new(FieldKey::new(key).map_err(invalid)?, comparison_of(comparison), value).map_err(invalid)
}

const fn comparison_of(dto: LibraryComparisonDto) -> FieldComparison {
    match dto {
        LibraryComparisonDto::Present => FieldComparison::Present,
        LibraryComparisonDto::Absent => FieldComparison::Absent,
        LibraryComparisonDto::Equals => FieldComparison::Equals,
        LibraryComparisonDto::Contains => FieldComparison::Contains,
        LibraryComparisonDto::LessThan => FieldComparison::LessThan,
        LibraryComparisonDto::GreaterThan => FieldComparison::GreaterThan,
        LibraryComparisonDto::WithinDays => FieldComparison::WithinDays,
    }
}

const fn comparison_dto(comparison: FieldComparison) -> LibraryComparisonDto {
    match comparison {
        FieldComparison::Present => LibraryComparisonDto::Present,
        FieldComparison::Absent => LibraryComparisonDto::Absent,
        FieldComparison::Equals => LibraryComparisonDto::Equals,
        FieldComparison::Contains => LibraryComparisonDto::Contains,
        FieldComparison::LessThan => LibraryComparisonDto::LessThan,
        FieldComparison::GreaterThan => LibraryComparisonDto::GreaterThan,
        FieldComparison::WithinDays => LibraryComparisonDto::WithinDays,
    }
}

const fn field_type_of(dto: FieldTypeDto) -> FieldType {
    match dto {
        FieldTypeDto::Text => FieldType::Text,
        FieldTypeDto::Number => FieldType::Number,
        FieldTypeDto::Money => FieldType::Money,
        FieldTypeDto::Date => FieldType::Date,
        FieldTypeDto::Boolean => FieldType::Boolean,
    }
}

const fn value_type_of(value: &FieldValue) -> FieldTypeDto {
    match value {
        FieldValue::Text(_) => FieldTypeDto::Text,
        FieldValue::Number(_) => FieldTypeDto::Number,
        FieldValue::Money { .. } => FieldTypeDto::Money,
        FieldValue::Date(_) => FieldTypeDto::Date,
        FieldValue::Flag(_) => FieldTypeDto::Boolean,
    }
}

impl From<DocumentKind> for LibraryKindDto {
    fn from(kind: DocumentKind) -> Self {
        match kind {
            DocumentKind::Pdf => Self::Pdf,
            DocumentKind::Image => Self::Image,
            DocumentKind::Office => Self::Office,
            DocumentKind::Other => Self::Other,
        }
    }
}

impl From<LibraryKindDto> for DocumentKind {
    fn from(kind: LibraryKindDto) -> Self {
        match kind {
            LibraryKindDto::Pdf => Self::Pdf,
            LibraryKindDto::Image => Self::Image,
            LibraryKindDto::Office => Self::Office,
            LibraryKindDto::Other => Self::Other,
        }
    }
}

impl From<LibraryGroupKeyDto> for GroupKey {
    fn from(key: LibraryGroupKeyDto) -> Self {
        match key {
            LibraryGroupKeyDto::Month => Self::Month,
            LibraryGroupKeyDto::Category => Self::Category,
            LibraryGroupKeyDto::Sender => Self::Sender,
        }
    }
}

impl From<LibrarySortColumnDto> for SortColumn {
    fn from(column: LibrarySortColumnDto) -> Self {
        match column {
            LibrarySortColumnDto::Name => Self::Name,
            LibrarySortColumnDto::Category => Self::Category,
            LibrarySortColumnDto::Correspondent => Self::Correspondent,
            LibrarySortColumnDto::Date => Self::Date,
            LibrarySortColumnDto::Amount => Self::Amount,
        }
    }
}

fn invalid(error: impl std::fmt::Display) -> IpcError {
    IpcError::invalid(error.to_string())
}

/// # Errors
/// `invalid_input` when `raw` is not a relative path inside the library.
pub fn parse_path(raw: &str) -> Result<RelativePath, IpcError> {
    RelativePath::of(raw).map_err(invalid)
}

/// # Errors
/// `invalid_input` for a blank id.
pub fn parse_category_id(raw: &str) -> Result<CategoryId, IpcError> {
    CategoryId::new(raw).map_err(invalid)
}

/// # Errors
/// `invalid_input` for a blank or too long tag.
pub fn parse_tag(raw: &str) -> Result<TagName, IpcError> {
    TagName::of(raw).map_err(invalid)
}

/// # Errors
/// `invalid_input` for a blank id.
pub fn parse_saved_search_id(raw: &str) -> Result<SavedSearchId, IpcError> {
    SavedSearchId::new(raw).map_err(invalid)
}

/// # Errors
/// `invalid_input` for a blank id.
pub fn parse_document_ids(raw: &[String]) -> Result<Vec<DocumentId>, IpcError> {
    raw.iter().map(|id| DocumentId::new(id.as_str()).map_err(invalid)).collect()
}
