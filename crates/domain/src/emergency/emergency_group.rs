/// The section of the emergency overview an item is listed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmergencyGroup {
    Contacts,
    Insurance,
    Accounts,
    Contracts,
    Personal,
    Other,
}
