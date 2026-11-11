/// What the user does with a rebuilt PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RebuildChoice {
    KeepOriginal,
    ReplaceWithRebuild,
    KeepBoth,
}
