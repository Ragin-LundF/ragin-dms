/// Files a sync client keeps only in the cloud. Reading one downloads it, so a scan reports it under its real
/// name without reading it.
pub struct CloudPlaceholders;

const ICLOUD_SUFFIX: &str = ".icloud";

/// macOS `SF_DATALESS`: the file's content lives only in the cloud.
#[cfg(target_os = "macos")]
const SF_DATALESS: u32 = 0x4000_0000;

/// Windows `FILE_ATTRIBUTE_OFFLINE`, `FILE_ATTRIBUTE_RECALL_ON_OPEN` and `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`.
#[cfg(windows)]
const NOT_LOCAL_ATTRIBUTES: u32 = 0x1000 | 0x4_0000 | 0x40_0000;

impl CloudPlaceholders {
    /// iCloud Drive stores a file that is not downloaded as `.<name>.icloud`; returns `<name>`, or `None`.
    #[must_use]
    pub fn real_name_of(name: &str) -> Option<&str> {
        let real = name.strip_prefix('.')?.strip_suffix(ICLOUD_SUFFIX)?;
        (!real.is_empty()).then_some(real)
    }

    /// Whether a file that exists under its real name is only a placeholder whose content a read would download.
    #[must_use]
    pub fn is_not_downloaded(metadata: &std::fs::Metadata) -> bool {
        #[cfg(target_os = "macos")]
        {
            use std::os::macos::fs::MetadataExt;
            metadata.st_flags() & SF_DATALESS != 0
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & NOT_LOCAL_ATTRIBUTES != 0
        }
        #[cfg(not(any(target_os = "macos", windows)))]
        {
            let _ = metadata;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_dot_name_dot_icloud_is_a_placeholder() {
        assert_eq!(CloudPlaceholders::real_name_of(".a.pdf.icloud"), Some("a.pdf"));
        assert_eq!(CloudPlaceholders::real_name_of("a.pdf.icloud"), None);
        assert_eq!(CloudPlaceholders::real_name_of("..icloud"), None);
        assert_eq!(CloudPlaceholders::real_name_of(".icloud"), None);
    }
}
