use crate::field::FieldKey;

/// A value of a suggestion that was read from the e-invoice (`ZUGFeRD`, Factur-X, `XRechnung`) inside the PDF instead
/// of being guessed; the review names that source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EInvoicePart {
    Correspondent,
    DocumentDate,
    Field(FieldKey),
}
