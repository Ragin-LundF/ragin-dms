//! What the library screen sends: its view parameters, and their parsing into checked domain values.

use dms_domain::document::{DocumentId, DocumentKind};
use dms_domain::library::RelativePath;
use dms_domain::query::{DateRange, SavedSearchId};
use dms_domain::taxonomy::{CategoryId, TagName};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::dto::IpcError;
use crate::features::library::model::{FacetFilter, GroupKey, Sort, SortColumn, View, ViewParams, YearFilter};

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
    Folder {
        path: String,
    },
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
    fn parse(&self) -> Result<FacetFilter, IpcError> {
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
            Self::Folder { path } => FacetFilter::Folder(parse_path(path)?),
        })
    }

    pub fn of(filter: &FacetFilter) -> Self {
        match filter {
            FacetFilter::Category(id) => Self::Category { category_id: id.as_ref().map(|id| id.value().to_owned()) },
            FacetFilter::Tag(tag) => Self::Tag { tag: tag.value().to_owned() },
            FacetFilter::Year(year) => Self::Year { year: year.year, and_earlier: year.and_earlier },
            FacetFilter::Correspondent(name) => Self::Correspondent { name: name.clone() },
            FacetFilter::Type(kind) => Self::Type { document_kind: (*kind).into() },
            FacetFilter::Folder(path) => Self::Folder { path: path.value().to_owned() },
        }
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
