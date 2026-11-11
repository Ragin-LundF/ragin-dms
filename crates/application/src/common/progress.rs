/// The hook of a long operation: it reports each step and can be asked to stop.
///
/// Kotlin passed `onProgress` callbacks and stopped long runs by cancelling the coroutine. Here an
/// operation checks [`Progress::is_cancelled`] between its items; a cancelled run stops before the next item
/// and returns the outcome of what it did so far. A plain closure `|step: &P| ...` is a `Progress<P>` that is
/// never cancelled.
pub trait Progress<P>: Sync {
    fn report(&self, _progress: &P) {}

    fn is_cancelled(&self) -> bool {
        false
    }
}

impl<P, F: Fn(&P) + Sync> Progress<P> for F {
    fn report(&self, progress: &P) {
        self(progress);
    }
}

/// Reports nothing and never cancels.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoProgress;

impl<P> Progress<P> for NoProgress {}
