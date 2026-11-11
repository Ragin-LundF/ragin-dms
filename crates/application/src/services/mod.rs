//! The composition of every use case from a set of adapters (Kotlin's `AppServices` and `AppContainer`). The
//! Tauri commands call through [`AppServices`].

mod app_ports;
mod app_services;
pub mod bundle;

pub use app_ports::AppPorts;
pub use app_services::AppServices;
pub use bundle::{
    AiServices, AskServices, DocumentServices, LibraryServices, OrganizeServices, PdfServices, SyncServices,
};
