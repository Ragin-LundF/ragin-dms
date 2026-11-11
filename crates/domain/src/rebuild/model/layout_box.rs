/// A box on a page in units of 0..1000 of the page width and height, origin top left. No pixel sizes cross the
/// ports; each side converts to its own image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutBox {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl LayoutBox {
    pub const SCALE: f64 = 1000.0;
}
