use crate::macros::non_blank_id;

non_blank_id!(
    /// Where a library folder lives on this device, as an absolute path in the device's own syntax.
    ///
    /// Locations differ per device and are therefore part of the device settings, never of the catalog.
    FolderLocation,
    "Folder location must not be blank"
);
