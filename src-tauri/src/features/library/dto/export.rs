//! What the export dialog sends: which documents and how to lay them out.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::params::LibraryParamsDto;

/// Which documents an export takes; resolved to one query when the export runs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryExportScopeDto {
    /// What the library shows with these parameters, its search text included.
    View {
        params: LibraryParamsDto,
    },
    SmartFolder {
        id: String,
    },
    /// Tagged "Tax" (the library's own spelling) and dated in `year`.
    TaxYear {
        year: i32,
    },
    Correspondent {
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)] // four independent switches, as in Kotlin
pub struct LibraryExportOptionsDto {
    pub flatten: bool,
    pub rename_to_title: bool,
    pub index: bool,
    pub zip: bool,
}
