/// What a region of a rebuilt page is, for the comparison overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionKind {
    Text,
    KeptLine,
    Image,
}
