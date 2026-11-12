use dms_domain::document::DocumentId;
use dms_domain::tax::{TaxArea, TaxItemId};

/// What the user changes on a tax year's checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaxItemChange {
    /// A new item of the user's own, which a rebuild never removes.
    Add {
        area: TaxArea,
        label: String,
        correspondent: Option<String>,
    },
    Rename {
        item_id: TaxItemId,
        label: String,
    },
    /// Marks the item "not needed this year", or with `false` needed again.
    SetNotNeeded {
        item_id: TaxItemId,
        not_needed: bool,
    },
    /// "I have it": links the document to the item and assigns it to the item's year and area.
    LinkDocument {
        item_id: TaxItemId,
        document_id: DocumentId,
    },
}
