use indexmap::IndexMap;

use crate::catalog::dto::{extensible, only_true, owned};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The term end, renewal and notice of a contract; `cancellableAnyTime` only when true. `NON_NULL`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContractTermsDto {
    pub term_end: Option<String>,
    pub renews_by_months: Option<i64>,
    pub notice_count: Option<i64>,
    pub notice_unit: Option<String>,
    pub cancellable_any_time: bool,
    pub unknown: IndexMap<String, Node>,
}

extensible!(ContractTermsDto);

const KNOWN: [&str; 5] = ["termEnd", "renewsByMonths", "noticeCount", "noticeUnit", "cancellableAnyTime"];

impl ContractTermsDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            term_end: owned(fields.opt_text("termEnd")?),
            renews_by_months: fields.opt_i64("renewsByMonths")?,
            notice_count: fields.opt_i64("noticeCount")?,
            notice_unit: owned(fields.opt_text("noticeUnit")?),
            cancellable_any_time: fields.bool_or("cancellableAnyTime", false)?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .opt_text("termEnd", self.term_end.as_deref())
            .opt_node("renewsByMonths", self.renews_by_months.map(Node::int))
            .opt_node("noticeCount", self.notice_count.map(Node::int))
            .opt_text("noticeUnit", self.notice_unit.as_deref())
            .opt_node("cancellableAnyTime", only_true(self.cancellable_any_time))
            .unknown(&self.unknown)
            .build()
    }
}
