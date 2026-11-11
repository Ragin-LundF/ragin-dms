use crate::rebuild::LayoutBox;

/// An area that is no printed text: a logo, stamp, signature, photo or code, with the AI's [`Self::kind`].
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutGraphic {
    pub kind: String,
    pub bounds: LayoutBox,
}
