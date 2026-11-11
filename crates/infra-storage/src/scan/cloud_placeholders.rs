/// Files a sync client keeps only in the cloud. Reading one downloads it, so a scan reports it under its real
/// name without reading it.
pub struct CloudPlaceholders;

const ICLOUD_SUFFIX: &str = ".icloud";

impl CloudPlaceholders {
    /// iCloud Drive stores a file that is not downloaded as `.<name>.icloud`; returns `<name>`, or `None`.
    #[must_use]
    pub fn real_name_of(name: &str) -> Option<&str> {
        let real = name.strip_prefix('.')?.strip_suffix(ICLOUD_SUFFIX)?;
        (!real.is_empty()).then_some(real)
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
