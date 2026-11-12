use super::*;

/// A synthetic mail: HTML body with an inline picture, two PDF attachments and an RFC 2047 encoded subject.
pub(crate) const MAIL: &str = "From: =?UTF-8?Q?Stadtwerke_N=C3=B6rdlingen?= <rechnung@stadtwerke.example>\r
To: Erika Beispiel <erika@example.org>\r
Subject: =?UTF-8?B?SWhyZSBSZWNobnVuZyBmw7xyIE9rdG9iZXI=?=\r
Date: Mon, 3 Nov 2026 23:30:00 +0100\r
MIME-Version: 1.0\r
Content-Type: multipart/mixed; boundary=\"outer\"\r
\r
--outer\r
Content-Type: multipart/related; boundary=\"inner\"\r
\r
--inner\r
Content-Type: text/html; charset=utf-8\r
\r
<html><body><p>Guten Tag,</p><p>anbei Ihre <b>Rechnung</b>.</p><img src=\"cid:logo\"></body></html>\r
--inner\r
Content-Type: image/png\r
Content-ID: <logo>\r
Content-Disposition: inline\r
Content-Transfer-Encoding: base64\r
\r
iVBORw0KGgo=\r
--inner--\r
--outer\r
Content-Type: application/pdf; name=\"Rechnung.pdf\"\r
Content-Disposition: attachment; filename=\"Rechnung.pdf\"\r
Content-Transfer-Encoding: base64\r
\r
JVBERi0xLjQgcmVjaG51bmc=\r
--outer\r
Content-Type: application/pdf\r
Content-Disposition: attachment; filename=\"=?UTF-8?Q?Gesch=C3=A4ftsbedingungen.pdf?=\"\r
Content-Transfer-Encoding: base64\r
\r
JVBERi0xLjQgYWdi\r
--outer--\r
";

#[test]
fn a_mail_is_read_with_decoded_header_html_body_as_text_and_its_two_attachments() {
    let mail = parse_mail(MAIL.as_bytes()).expect("a mail");

    assert_eq!(mail.from_name.as_deref(), Some("Stadtwerke Nördlingen"));
    assert_eq!(mail.from_address.as_deref(), Some("rechnung@stadtwerke.example"));
    assert_eq!(mail.to, vec!["Erika Beispiel <erika@example.org>".to_owned()]);
    assert_eq!(mail.subject.as_deref(), Some("Ihre Rechnung für Oktober"));
    assert_eq!(mail.sent_at.map(|date| date.to_rfc3339()).as_deref(), Some("2026-11-03T23:30:00+01:00"));
    assert!(mail.body.contains("Guten Tag,") && mail.body.contains("anbei Ihre Rechnung."), "{}", mail.body);
    assert!(!mail.body.contains('<'), "no HTML left: {}", mail.body);
    let names: Vec<&str> = mail.attachments.iter().map(|part| part.name.as_str()).collect();
    assert_eq!(names, vec!["Rechnung.pdf", "Geschäftsbedingungen.pdf"]);
    assert_eq!(mail.attachments[0].content, b"%PDF-1.4 rechnung");
}

#[test]
fn text_that_is_no_mail_is_an_unsupported_format() {
    assert!(matches!(parse_mail(b"just some words"), Err(PdfError::UnsupportedFormat(_))));
    assert!(matches!(parse_mail(b""), Err(PdfError::UnsupportedFormat(_))));
}

#[test]
fn an_attachment_is_written_once_and_never_over_an_existing_file() {
    let temp = tempfile::tempdir().expect("temp");
    let target = temp.path().join("Rechnung.pdf");

    let facts = write_attachment(b"%PDF", &target).expect("written");

    assert_eq!(facts.size, 4);
    assert!(matches!(write_attachment(b"other", &target), Err(PdfError::AlreadyExists)));
    assert_eq!(fs::read(&target).expect("read"), b"%PDF");
}
