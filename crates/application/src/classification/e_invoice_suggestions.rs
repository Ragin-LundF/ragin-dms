use dms_domain::bill::EInvoice;
use dms_domain::catalog::Catalog;
use dms_domain::classification::{ClassificationSuggestion, EInvoicePart};
use dms_domain::document::Document;
use dms_domain::field::{FieldKey, FieldPresets, FieldRole, FieldValue};

/// What an e-invoice adds to a suggestion: its exact payment data in place of guessed values, each value marked as
/// read from the e-invoice. The payee becomes the correspondent, the issue date the document date; amount, due date,
/// IBAN and payment reference (or else the invoice number) fill the bill fields. Missing standard fields are
/// defined when the suggestion is approved.
pub struct EInvoiceSuggestions;

impl EInvoiceSuggestions {
    /// A suggestion made from the e-invoice alone, without the AI.
    #[must_use]
    pub fn of(document: &Document, e_invoice: &EInvoice, catalog: &Catalog) -> ClassificationSuggestion {
        let base = ClassificationSuggestion::empty(document.id.clone(), ClassificationSuggestion::E_INVOICE_MODEL);
        Self::over(base, e_invoice, catalog)
    }

    /// `suggestion` with every value the e-invoice has in place of the suggested one; the model names the
    /// e-invoice too.
    #[must_use]
    pub fn over(
        suggestion: ClassificationSuggestion,
        e_invoice: &EInvoice,
        catalog: &Catalog,
    ) -> ClassificationSuggestion {
        let mut suggestion = suggestion;
        let mut parts = Vec::new();
        if let Some(seller) = &e_invoice.seller {
            suggestion.correspondent = Some(seller.clone());
            parts.push(EInvoicePart::Correspondent);
        }
        if let Some(issued) = e_invoice.issue_date {
            suggestion.document_date = Some(issued);
            parts.push(EInvoicePart::DocumentDate);
        }
        let reference = e_invoice.payment_reference.as_ref().or(e_invoice.invoice_number.as_ref());
        let fields = [
            (
                FieldPresets::amount().key,
                e_invoice
                    .amount
                    .clone()
                    .map(|money| FieldValue::Money { amount: money.amount, currency: money.currency }),
            ),
            (due_date_key(catalog), e_invoice.due_date.map(FieldValue::Date)),
            (FieldPresets::iban().key, e_invoice.iban.as_ref().map(|iban| FieldValue::Text(iban.value().to_owned()))),
            (FieldPresets::payment_reference().key, reference.map(|reference| FieldValue::Text(reference.clone()))),
        ];
        for (key, value) in fields {
            let Some(value) = value else { continue };
            suggestion.fields.insert(key.clone(), value);
            parts.push(EInvoicePart::Field(key));
        }
        if parts.is_empty() {
            return suggestion;
        }
        if suggestion.model != ClassificationSuggestion::E_INVOICE_MODEL {
            suggestion.model = format!("{} + {}", suggestion.model, ClassificationSuggestion::E_INVOICE_MODEL);
        }
        suggestion.e_invoice = parts;
        suggestion
    }
}

/// The library's first due date field, or the standard one.
fn due_date_key(catalog: &Catalog) -> FieldKey {
    catalog
        .settings()
        .field_definitions
        .iter()
        .find(|definition| definition.role == Some(FieldRole::DueDate))
        .map_or_else(|| FieldPresets::due_date().key, |definition| definition.key.clone())
}
