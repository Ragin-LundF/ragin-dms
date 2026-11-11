use crate::library::Dashboard;

#[derive(Debug, Clone, PartialEq)]
pub enum DashboardOutcome {
    Built(Dashboard),
    LibraryNotOpen,
}
