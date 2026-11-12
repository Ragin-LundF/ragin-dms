//! The payment data of an e-invoice embedded in a PDF (`ZUGFeRD`, Factur-X, `XRechnung` in the UN/CEFACT Cross
//! Industry Invoice syntax). Only the values a bill needs are read; anything else in the XML is ignored.

use std::path::Path;

use dms_domain::bill::{EInvoice, Iban};
use dms_domain::field::{CurrencyCode, Decimal};
use dms_domain::money::Money;
use dms_domain::text;
use quick_xml::events::Event;

use crate::PdfEngine;

/// The attachment names of the e-invoice standards, in the order they are looked for.
const NAMES: [&str; 4] = ["factur-x.xml", "zugferd-invoice.xml", "ZUGFeRD-invoice.xml", "xrechnung.xml"];

/// The e-invoice of the PDF at `file`; `None` when it has none or it cannot be read.
pub(crate) fn read(engine: PdfEngine, file: &Path) -> Option<EInvoice> {
    // A PDF that cannot be opened has no e-invoice to offer; reading the document reports that problem.
    let xml = engine.attachment(file, &NAMES).ok()??;
    parse(&xml)
}

/// The values of a CII invoice; `None` for XML that is broken or holds none of them (such as UBL).
pub(crate) fn parse(xml: &[u8]) -> Option<EInvoice> {
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut value = String::new();
    let mut found = Found::default();
    loop {
        match reader.read_event_into(&mut buffer).ok()? {
            Event::Start(element) => {
                path.push(element.local_name().as_ref().to_owned());
                value.clear();
            }
            Event::End(_) => {
                found.take(&path, text::trim(&value));
                value.clear();
                path.pop();
            }
            Event::Text(part) => value.push_str(&part.into_inner()),
            Event::GeneralRef(reference) => {
                value.push_str(&quick_xml::escape::unescape(&format!("&{};", reference.into_inner())).ok()?);
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    found.into_e_invoice()
}

#[derive(Default)]
struct Found {
    seller: Option<String>,
    invoice_number: Option<String>,
    issue_date: Option<String>,
    due_date: Option<String>,
    amount: Option<String>,
    currency: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
    payment_reference: Option<String>,
}

impl Found {
    /// Keeps `value` when the element path ends like one of the wanted values; the first occurrence wins.
    fn take(&mut self, path: &[String], value: &str) {
        if value.is_empty() {
            return;
        }
        let ends_with = |tail: &[&str]| path.len() >= tail.len() && path[path.len() - tail.len()..] == *tail;
        let slot = if ends_with(&["ExchangedDocument", "ID"]) {
            &mut self.invoice_number
        } else if ends_with(&["ExchangedDocument", "IssueDateTime", "DateTimeString"]) {
            &mut self.issue_date
        } else if ends_with(&["SellerTradeParty", "Name"]) {
            &mut self.seller
        } else if ends_with(&["SpecifiedTradePaymentTerms", "DueDateDateTime", "DateTimeString"]) {
            &mut self.due_date
        } else if ends_with(&["SpecifiedTradeSettlementHeaderMonetarySummation", "DuePayableAmount"]) {
            &mut self.amount
        } else if ends_with(&["ApplicableHeaderTradeSettlement", "InvoiceCurrencyCode"]) {
            &mut self.currency
        } else if ends_with(&["PayeePartyCreditorFinancialAccount", "IBANID"]) {
            &mut self.iban
        } else if ends_with(&["PayeeSpecifiedCreditorFinancialInstitution", "BICID"]) {
            &mut self.bic
        } else if ends_with(&["ApplicableHeaderTradeSettlement", "PaymentReference"]) {
            &mut self.payment_reference
        } else {
            return;
        };
        slot.get_or_insert_with(|| value.to_owned());
    }

    fn into_e_invoice(self) -> Option<EInvoice> {
        let amount = match (self.amount.as_deref().and_then(Decimal::parse), self.currency) {
            (Some(amount), Some(currency)) => {
                CurrencyCode::new(currency).ok().map(|currency| Money { amount, currency })
            }
            _ => None,
        };
        let e_invoice = EInvoice {
            seller: self.seller,
            invoice_number: self.invoice_number,
            issue_date: self.issue_date.as_deref().and_then(date_102),
            due_date: self.due_date.as_deref().and_then(date_102),
            amount,
            // A malformed IBAN is left out rather than offered for a payment.
            iban: self.iban.as_deref().and_then(|iban| Iban::parse(iban).ok()),
            bic: self.bic,
            payment_reference: self.payment_reference,
        };
        (e_invoice != EInvoice::default()).then_some(e_invoice)
    }
}

/// A date in the CII format 102, `YYYYMMDD`.
fn date_102(raw: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(raw.get(..8)?, "%Y%m%d").ok()
}
