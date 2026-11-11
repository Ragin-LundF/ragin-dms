//! Wiring of the `pdf` use cases: they need the PDF fakes, which `TestLibrary` does not hold, so they are wired
//! on `PdfKit` in `tests/spec/pdf/kit.rs`.
