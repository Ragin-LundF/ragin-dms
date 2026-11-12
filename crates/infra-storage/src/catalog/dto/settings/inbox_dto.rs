use indexmap::IndexMap;

use crate::catalog::dto::{extensible, only_true};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The inbox settings; every flag of 1.x is written, so the file says what is switched on. The switches added in 2.1
/// are written only when on, so a catalog that does not use them stays as 2.0 wrote it.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)] // the flags of the file format
pub struct InboxDto {
    pub enabled: bool,
    pub path: String,
    pub ai_analysis: bool,
    pub ai_target_folder: bool,
    pub auto_analyze: bool,
    pub propose_splits: bool,
    pub convert_emails: bool,
    pub keep_email_originals: bool,
    pub unknown: IndexMap<String, Node>,
}

extensible!(InboxDto);

const KNOWN: [&str; 8] = [
    "enabled",
    "path",
    "aiAnalysis",
    "aiTargetFolder",
    "autoAnalyze",
    "proposeSplits",
    "convertEmails",
    "keepEmailOriginals",
];

impl InboxDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            enabled: fields.bool_or("enabled", false)?,
            path: fields.opt_text("path")?.unwrap_or("Inbox").to_owned(),
            ai_analysis: fields.bool_or("aiAnalysis", true)?,
            ai_target_folder: fields.bool_or("aiTargetFolder", true)?,
            auto_analyze: fields.bool_or("autoAnalyze", false)?,
            propose_splits: fields.bool_or("proposeSplits", false)?,
            convert_emails: fields.bool_or("convertEmails", false)?,
            keep_email_originals: fields.bool_or("keepEmailOriginals", false)?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .bool("enabled", self.enabled)
            .text("path", &self.path)
            .bool("aiAnalysis", self.ai_analysis)
            .bool("aiTargetFolder", self.ai_target_folder)
            .bool("autoAnalyze", self.auto_analyze)
            .opt_node("proposeSplits", only_true(self.propose_splits))
            .opt_node("convertEmails", only_true(self.convert_emails))
            .opt_node("keepEmailOriginals", only_true(self.keep_email_originals))
            .unknown(&self.unknown)
            .build()
    }
}
