//! The use cases the Ask screen calls (`AppServices`).

use dms_application::ask::{AskOutcome, IndexProgress};
use dms_application::common::Progress;
use dms_domain::query::DocumentQuery;

use crate::context::AppContext;
use crate::dto::IpcError;

pub fn ask_library(ctx: &AppContext, question: &str, scope: &DocumentQuery) -> Result<AskOutcome, IpcError> {
    Ok(ctx.require_services()?.ask.ask_library.run(question, scope))
}

pub fn index_library(
    ctx: &AppContext,
    progress: &dyn Progress<IndexProgress>,
) -> Result<Option<IndexProgress>, IpcError> {
    Ok(ctx.require_services()?.ask.index_library.run(progress))
}
