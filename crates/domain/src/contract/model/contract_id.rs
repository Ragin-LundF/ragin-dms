use crate::macros::non_blank_id;

non_blank_id!(
    /// Identifies a contract; a UUID chosen when the contract is created.
    ContractId,
    "Contract id must not be blank"
);
