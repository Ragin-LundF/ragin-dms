use dms_domain::library::RelativePath;

/// Where the emergency overview is saved: as a catalogued document in a folder of the library, or as a file named
/// `name` in `directory`, an absolute path of this device outside the library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OverviewTarget {
    Library { folder: RelativePath },
    Outside { directory: String, name: String },
}
