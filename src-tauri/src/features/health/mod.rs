//! health: the archive check (Settings › Archive health, the dashboard card, "Check archive now"), its automatic
//! start when due, repairs through the dashboard actions, and the catalog snapshots with compare and restore.

pub mod check;
pub mod commands;
pub mod dto;
mod view;
