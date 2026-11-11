use dms_domain::rebuild::PageLayout;

use crate::port::pdf::PreparedPage;

/// What the rebuild writes for one page.
#[derive(Debug, Clone)]
pub enum PageInput {
    /// A page the AI read: rebuilt from `layout` on the straightened `prepared` page.
    Read { prepared: PreparedPage, layout: PageLayout },
    /// A page the AI could not read: page `page_index` of the original stays as it is.
    Failed { page_index: usize },
}
