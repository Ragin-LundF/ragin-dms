use dms_domain::change::FileFacts;
use dms_domain::library::RelativePath;

/// Follows the scan of one folder. A scan first lists the folder, then reads the files one by one;
/// [`Self::file_done`] counts up to the total that [`Self::counted`] announced. Both may be called from the
/// scanner's worker threads.
pub trait ScanListener: Sync {
    /// The folder holds `total` files, which are read next.
    fn counted(&self, _total: usize) {}

    /// `done` files are read; the last was `path` with `facts`. `hashed` tells whether its content was read in this
    /// scan rather than taken from the scan cache.
    fn file_done(&self, _done: usize, _path: &RelativePath, _facts: &FileFacts, _hashed: bool) {}

    /// `true` to stop the scan before the next file; the Rust stand-in for cancelling the scanning coroutine.
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Follows nothing and never cancels; Kotlin's `ScanListener.NONE`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoScanListener;

impl ScanListener for NoScanListener {}
