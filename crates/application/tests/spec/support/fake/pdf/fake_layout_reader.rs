use std::collections::HashMap;

use dms_application::port::ai::{LayoutOutcome, LineOutcome, PageImage, PageLayoutReader};
use dms_domain::rebuild::PageLayout;

use crate::support::Shared;

type BeforeLayout = Box<dyn FnMut(usize) + Send>;

/// Answers with the scripted `layouts` by page number, an empty page otherwise, and with `lines` by the crop's image
/// text (its box). `before_layout` runs before each answer, so a test can act while a page is being read.
#[derive(Default)]
pub struct FakeLayoutReader {
    pub layouts: Shared<HashMap<usize, LayoutOutcome>>,
    pub lines: Shared<HashMap<String, LineOutcome>>,
    before_layout: Shared<Option<BeforeLayout>>,
}

impl FakeLayoutReader {
    pub fn set_before_layout(&self, hook: impl FnMut(usize) + Send + 'static) {
        self.before_layout.set(Some(Box::new(hook)));
    }
}

impl PageLayoutReader for FakeLayoutReader {
    fn read_layout(&self, page: &PageImage) -> LayoutOutcome {
        let text = String::from_utf8_lossy(&page.bytes);
        let number: usize = text.trim_start_matches("page ").parse().expect("a prepared page image");
        if let Some(hook) = self.before_layout.lock().as_mut() {
            hook(number);
        }
        self.layouts.lock().get(&number).cloned().unwrap_or_else(|| LayoutOutcome::Read(PageLayout::default()))
    }

    fn read_line(&self, line: &PageImage) -> LineOutcome {
        let text = String::from_utf8_lossy(&line.bytes).into_owned();
        self.lines.lock().get(&text).cloned().unwrap_or_else(|| LineOutcome::Read(String::new()))
    }
}
