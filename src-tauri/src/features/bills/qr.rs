//! The payment code as SVG markup, ready to show in the window.

use dms_domain::bill::EpcPayment;
use fast_qr::ECL;
use fast_qr::convert::svg::SvgBuilder;
use fast_qr::qr::QRBuilder;

/// The EPC QR code of `payment` as SVG, with the error correction level M that EPC069-12 asks for; `None` if the
/// payload does not fit a QR code, which the length limits of [`EpcPayment`] rule out.
pub fn payment_svg(payment: &EpcPayment) -> Option<String> {
    let code = QRBuilder::new(payment.payload()).ecl(ECL::M).build().ok()?;
    Some(SvgBuilder::default().to_str(&code))
}

#[cfg(test)]
mod tests {
    use dms_domain::bill::Iban;
    use dms_domain::field::{CurrencyCode, Decimal};
    use dms_domain::money::Money;

    use super::*;

    #[test]
    fn a_payment_becomes_an_svg_qr_code() {
        let euros = Money {
            amount: Decimal::parse("86.40").expect("decimal"),
            currency: CurrencyCode::new("EUR").expect("currency"),
        };
        let iban = Iban::parse("DE89370400440532013000").expect("iban");
        let payment = EpcPayment::new("Sample Energy", iban, None, &euros, Some("KD 4711")).expect("payment");

        let svg = payment_svg(&payment).expect("an svg");

        assert!(svg.starts_with("<svg") && svg.trim_end().ends_with("</svg>"), "{svg}");
    }
}
