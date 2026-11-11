//! `dms://` serves document previews to the webview, read-only. Routes (after `dms://localhost/`):
//! - `thumb/<folder id>/<relative path>`: the first page of a file as a small picture (Kotlin `ThumbnailLoader`),
//!   for grids, lists, quick look, the inbox and the dashboard; also for files not in the catalog yet,
//! - `page/<document id>/<page index>?width=<px>`: one page of a catalogued PDF, for the PDF tools,
//! - `rebuilt/<rebuild id>/<page index>?width=<px>`: one page of a "Convert to real PDF" result being compared.
//!
//! The UI builds these URLs with `convertFileSrc(path, "dms")`, which picks the right form per platform.

use percent_encoding::percent_decode_str;
use std::sync::Arc;
use tauri::http::{Request, Response, StatusCode, header};

use dms_application::port::ai::PageImage;
use dms_application::preview::ThumbnailOutcome;
use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolderId, RelativePath};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::context::AppContext;

pub const SCHEME: &str = "dms";

/// A preview request, parsed from the URL; ids are checked by the previewer, never used as paths.
#[derive(Debug, PartialEq, Eq)]
pub enum PreviewRequest {
    Thumbnail { folder: String, path: String },
    Page { document: String, page: u32, width: Option<u32> },
    Rebuilt { rebuild: String, page: u32, width: Option<u32> },
}

/// Kotlin rendered page pictures at this longest side when no width was asked for.
const DEFAULT_SIDE: u32 = 1170;

#[allow(clippy::needless_pass_by_value)] // signature required by Tauri's protocol registration
pub fn handle<R: Runtime>(context: UriSchemeContext<'_, R>, request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let uri = request.uri();
    let parsed = parse(uri.path(), uri.query().unwrap_or(""));
    let ctx = context.app_handle().try_state::<Arc<AppContext>>().map(|state| Arc::clone(state.inner()));
    // shortcut: one thread per picture; PDFium serializes rendering anyway. Use a pool if a grid of hundreds stalls.
    std::thread::spawn(move || {
        let response = match (parsed, ctx) {
            (None, _) => status(StatusCode::BAD_REQUEST),
            (Some(_), None) => status(StatusCode::SERVICE_UNAVAILABLE),
            (Some(request), Some(ctx)) => serve(&ctx, &request)
                .map_or_else(|| status(StatusCode::NOT_FOUND), |picture| image(picture.bytes, &picture.mime_type)),
        };
        responder.respond(response);
    });
}

fn serve(ctx: &AppContext, request: &PreviewRequest) -> Option<PageImage> {
    let services = ctx.services()?;
    match request {
        PreviewRequest::Thumbnail { folder, path } => {
            let folder = LibraryFolderId::new(folder.as_str()).ok()?;
            let path = RelativePath::parse(path)?;
            match services.library.thumbnails.run(&folder, &path) {
                ThumbnailOutcome::Rendered(picture) => Some(picture),
                ThumbnailOutcome::NoPicture | ThumbnailOutcome::FolderNotFound => None,
            }
        }
        PreviewRequest::Page { document, page, width } => {
            let id = DocumentId::new(document.as_str()).ok()?;
            let path = ctx.catalog()?.documents().get(&id)?.path.clone();
            let folders = services.library.settings.load_library_folders().ok().flatten()?;
            let side = width.unwrap_or(DEFAULT_SIDE);
            services.pdf.editor.render_page(folders.primary(), &path, usize::try_from(*page).ok()?, side)
        }
        PreviewRequest::Rebuilt { rebuild, page, width } => {
            let rebuilt = crate::features::pdf::state::rebuilt_file(rebuild)?;
            services.pdf.rebuilder.render_page(&rebuilt, usize::try_from(*page).ok()?, width.unwrap_or(DEFAULT_SIDE))
        }
    }
}

pub fn parse(path: &str, query: &str) -> Option<PreviewRequest> {
    let decoded = percent_decode_str(path.trim_start_matches('/')).decode_utf8().ok()?;
    if let Some(rest) = decoded.strip_prefix("thumb/") {
        let (folder, path) = rest.split_once('/')?;
        // The library resolves the path inside the folder; `..`, empty and absolute segments never reach it.
        let safe = !path.is_empty() && path.split('/').all(|segment| !matches!(segment, "" | "." | ".."));
        return (valid_id(folder) && safe)
            .then(|| PreviewRequest::Thumbnail { folder: folder.to_owned(), path: path.to_owned() });
    }
    let mut parts = decoded.split('/');
    let request = match (parts.next()?, parts.next()?, parts.next()) {
        ("page", document, Some(page)) if valid_id(document) => {
            PreviewRequest::Page { document: document.to_owned(), page: page.parse().ok()?, width: width(query) }
        }
        ("rebuilt", rebuild, Some(page)) if valid_id(rebuild) => {
            PreviewRequest::Rebuilt { rebuild: rebuild.to_owned(), page: page.parse().ok()?, width: width(query) }
        }
        _ => return None,
    };
    parts.next().is_none().then_some(request)
}

fn width(query: &str) -> Option<u32> {
    query.split('&').find_map(|pair| pair.strip_prefix("width="))?.parse().ok()
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn image(bytes: Vec<u8>, content_type: &str) -> Response<Vec<u8>> {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=60")
        .body(bytes)
        .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR))
}

fn status(code: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = code;
    response
}

#[cfg(test)]
mod tests {
    use super::{PreviewRequest, parse};

    #[test]
    fn parses_thumbnails_and_pages() {
        assert_eq!(
            parse("/thumb/f-1/Tax/2024%20bill.pdf", ""),
            Some(PreviewRequest::Thumbnail { folder: "f-1".to_owned(), path: "Tax/2024 bill.pdf".to_owned() })
        );
        assert_eq!(
            parse("/page/abc/2", "width=800"),
            Some(PreviewRequest::Page { document: "abc".to_owned(), page: 2, width: Some(800) })
        );
    }

    #[test]
    fn refuses_paths_and_unknown_routes() {
        assert_eq!(parse("/thumb/f/..%2Fetc", ""), None);
        assert_eq!(parse("/thumb/f//etc", ""), None);
        assert_eq!(parse("/thumb/f", ""), None);
        assert_eq!(parse("/file/abc", ""), None);
        assert_eq!(parse("/page/abc/x", ""), None);
    }
}
