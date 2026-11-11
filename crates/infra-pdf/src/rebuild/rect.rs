use super::model::LayoutBox;

/// A pixel rectangle, as `java.awt.Rectangle`: a width or height of 0 or less is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect { x, y, width, height }
    }

    pub fn right(self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(self) -> i32 {
        self.y + self.height
    }

    pub fn is_empty(self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    pub fn center_y(self) -> f64 {
        f64::from(self.y) + f64::from(self.height) / 2.0
    }

    pub fn area(self) -> f64 {
        f64::from(self.width) * f64::from(self.height)
    }

    pub fn intersection(self, other: Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Rect::new(x, y, self.right().min(other.right()) - x, self.bottom().min(other.bottom()) - y)
    }

    pub fn intersects(self, other: Rect) -> bool {
        !self.intersection(other).is_empty()
    }
}

/// Converts boxes between 0..1000 of a page and pixels of its [`width`] x [`height`] image.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PixelBoxes {
    pub width: i32,
    pub height: i32,
}

impl PixelBoxes {
    /// The pixels of [`bounds`], at least one pixel and inside the image.
    pub fn to_pixels(self, bounds: LayoutBox) -> Rect {
        let x0 = pixels(bounds.left, self.width).clamp(0, self.width - 1);
        let y0 = pixels(bounds.top, self.height).clamp(0, self.height - 1);
        let x1 = pixels(bounds.right, self.width).clamp(x0 + 1, self.width);
        let y1 = pixels(bounds.bottom, self.height).clamp(y0 + 1, self.height);
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }

    pub fn to_layout(self, rect: Rect) -> LayoutBox {
        let (width, height) = (f64::from(self.width), f64::from(self.height));
        LayoutBox {
            left: f64::from(rect.x) * LayoutBox::SCALE / width,
            top: f64::from(rect.y) * LayoutBox::SCALE / height,
            right: f64::from(rect.right()) * LayoutBox::SCALE / width,
            bottom: f64::from(rect.bottom()) * LayoutBox::SCALE / height,
        }
    }
}

#[allow(clippy::cast_possible_truncation)] // truncates toward zero like Kotlin's toInt; saturates on nonsense
fn pixels(value: f64, size: i32) -> i32 {
    (value * f64::from(size) / LayoutBox::SCALE) as i32
}
