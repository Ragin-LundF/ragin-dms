//! Fakes of the PDF ports (`PdfEditor`, `OcrTool`, `PdfRebuilder`) and the layout reader, owned by the pdf tests.

mod fake_layout_reader;
mod fake_ocr;
mod fake_pdf_editor;
mod fake_pdf_rebuilder;

pub use fake_layout_reader::FakeLayoutReader;
pub use fake_ocr::FakeOcr;
pub use fake_pdf_editor::FakePdfEditor;
pub use fake_pdf_rebuilder::FakePdfRebuilder;
