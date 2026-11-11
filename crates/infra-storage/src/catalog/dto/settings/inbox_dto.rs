use indexmap::IndexMap;

use crate::catalog::dto::extensible;
use crate::yaml::{Fields, MapWriter, Node, YamlError};

/// The inbox settings; every flag is written, so the file says what is switched on.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)] // the flags of the file format
pub struct InboxDto {
    pub enabled: bool,
    pub path: String,
    pub ai_analysis: bool,
    pub ai_target_folder: bool,
    pub auto_analyze: bool,
    pub unknown: IndexMap<String, Node>,
}

extensible!(InboxDto);

const KNOWN: [&str; 5] = ["enabled", "path", "aiAnalysis", "aiTargetFolder", "autoAnalyze"];

impl InboxDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        Ok(Self {
            enabled: fields.bool_or("enabled", false)?,
            path: fields.opt_text("path")?.unwrap_or("Inbox").to_owned(),
            ai_analysis: fields.bool_or("aiAnalysis", true)?,
            ai_target_folder: fields.bool_or("aiTargetFolder", true)?,
            auto_analyze: fields.bool_or("autoAnalyze", false)?,
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
            .unknown(&self.unknown)
            .build()
    }
}
