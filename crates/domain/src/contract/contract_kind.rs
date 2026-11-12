/// What kind of contract it is; it decides the icon and what the app expects from it (bills for utilities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContractKind {
    Utility,
    Telecom,
    Insurance,
    Subscription,
    Membership,
    Loan,
    Other,
}
