//! Use cases and ports of ragin-dms, ported from the Kotlin module `dms-application`.
//!
//! Every use case is a struct holding `Arc`s of its ports and collaborators with one `run` method (Kotlin's
//! `operator fun invoke`). Kotlin's `suspend` I/O functions are plain blocking functions here: callers run use
//! cases on worker threads, and every port is `Send + Sync` so it can be shared as `Arc<dyn Port>`.
//! [`services::AppServices`] wires every use case from a set of adapters.

// Use cases return outcomes the caller must look at anyway, and panic only on invariants such as a generated id
// being blank; marking every `run` and documenting every `expect` would only add noise.
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

pub mod common;
pub mod port;

pub mod ask;
pub mod assistant;
pub mod catalog;
pub mod classification;
pub mod deadline;
pub mod document;
pub mod export;
pub mod folder;
pub mod inbox;
pub mod library;
pub mod pdf;
pub mod preview;
pub mod resolution;
pub mod restructure;
pub mod rule;
pub mod search;
pub mod services;
pub mod sidecar;
pub mod sync;
