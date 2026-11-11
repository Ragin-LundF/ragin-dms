//! The library's Tauri commands: parse the input, read the catalog or call one use case, map to DTOs.

pub mod bulk;
pub mod export;
pub mod folders;
mod operation;
pub mod saved_views;
pub mod taxonomy;
pub mod view;

use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId};

use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::library::model::Moment;

/// Now, in the zone the library shows dates in, from the services' clock.
fn moment(ctx: &AppContext) -> Result<Moment, IpcError> {
    let clock = &ctx.require_services()?.library.clock;
    Ok(Moment { now: clock.now(), zone: clock.zone() })
}

/// The catalogued documents of `ids`, in that order; ids the catalog no longer has are left out.
fn documents_of<'a>(catalog: &'a Catalog, ids: &[DocumentId]) -> Vec<&'a Document> {
    ids.iter().filter_map(|id| catalog.documents().get(id)).collect()
}
