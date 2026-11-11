//! Each feature wires its use cases on `TestLibrary` in its own file; `core` holds the ones other features need.

mod ask;
mod assistant;
mod catalog;
mod classification;
mod core;
mod deadline;
mod document;
mod export;
mod folder;
mod inbox;
mod library;
mod pdf;
mod preview;
mod resolution;
mod restructure;
mod rule;
mod search;
mod sync;
