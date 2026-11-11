//! Data the commands exchange with the UI. Domain types never cross the IPC boundary; each feature maps them here.

mod error;

pub use error::IpcError;
