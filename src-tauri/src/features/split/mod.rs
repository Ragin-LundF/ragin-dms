//! split: the "Split a stack" dialog. The AI proposes where each document of a scanned PDF starts, the user sets the
//! cuts, and every part waits for review. Shown only while "Use AI" is on.

pub mod commands;
pub mod dto;
mod view;
