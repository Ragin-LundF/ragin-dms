use std::path::PathBuf;

use dms_domain::text;

/// Where the tool keeps the state of this device: settings, journal, tasks, activity and scan caches. Never
/// inside a library folder. Deliberately the same folder as the Kotlin version, not a Tauri bundle identifier:
/// macOS `~/Library/Application Support/<name>`, Windows `%APPDATA%\<name>` (else `~/AppData/Roaming/<name>`),
/// elsewhere `$XDG_DATA_HOME/<name>` (else `~/.local/share/<name>`). A blank variable counts as unset.
pub struct AppDataDirectory;

impl AppDataDirectory {
    pub const APPLICATION_NAME: &'static str = "ragin-dms";

    /// The directory for `name` on this device, or `None` when the user has no home directory.
    #[must_use]
    pub fn for_application(name: &str) -> Option<PathBuf> {
        let home = std::env::home_dir()?;
        Some(Self::resolve(name, &home, std::env::consts::OS, |variable| std::env::var(variable).ok()))
    }

    fn resolve(name: &str, home: &std::path::Path, os: &str, environment: impl Fn(&str) -> Option<String>) -> PathBuf {
        let variable = |key: &str| environment(key).filter(|value| !text::is_blank(value)).map(PathBuf::from);
        let base = match os {
            "macos" => home.join("Library/Application Support"),
            "windows" => variable("APPDATA").unwrap_or_else(|| home.join("AppData/Roaming")),
            _ => variable("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share")),
        };
        base.join(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(os: &str, variables: &[(&str, &str)]) -> PathBuf {
        let lookup = |key: &str| variables.iter().find(|(name, _)| *name == key).map(|(_, value)| (*value).to_owned());
        AppDataDirectory::resolve("ragin-dms", std::path::Path::new("/home/user"), os, lookup)
    }

    #[test]
    fn each_platform_uses_the_folder_of_the_kotlin_version() {
        assert_eq!(resolved("macos", &[]), PathBuf::from("/home/user/Library/Application Support/ragin-dms"));
        assert_eq!(resolved("windows", &[("APPDATA", "/roaming")]), PathBuf::from("/roaming/ragin-dms"));
        assert_eq!(resolved("windows", &[]), PathBuf::from("/home/user/AppData/Roaming/ragin-dms"));
        assert_eq!(resolved("linux", &[("XDG_DATA_HOME", "/data")]), PathBuf::from("/data/ragin-dms"));
        assert_eq!(resolved("linux", &[("XDG_DATA_HOME", "  ")]), PathBuf::from("/home/user/.local/share/ragin-dms"));
    }
}
