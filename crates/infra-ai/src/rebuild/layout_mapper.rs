//! Maps the AI's answer to a layout in 0..1000 of the page. The prompt asks for 0..1000, which Qwen3-style models
//! give; Qwen2.5-style models answer in pixels of the image they saw. If any value is above 1000, the whole answer is
//! taken as pixels of the sent image.

use dms_domain::rebuild::{FontFamily, LayoutBox, LayoutGraphic, LayoutLine, PageLayout};

use super::layout_dto::LayoutAnswerDto;

const BOX_SIZE: usize = 4;
const DEFAULT_GRAPHIC: &str = "graphic";

pub(crate) fn to_layout(answer: LayoutAnswerDto, sent_width: u32, sent_height: u32) -> PageLayout {
    let lines: Vec<_> = answer.lines.into_iter().filter(|line| line.bbox.len() == BOX_SIZE).collect();
    let graphics: Vec<_> = answer.graphics.into_iter().filter(|graphic| graphic.bbox.len() == BOX_SIZE).collect();
    let pixels = lines
        .iter()
        .flat_map(|line| &line.bbox)
        .chain(graphics.iter().flat_map(|graphic| &graphic.bbox))
        .any(|&value| value > LayoutBox::SCALE);
    let scale = if pixels {
        (LayoutBox::SCALE / f64::from(sent_width), LayoutBox::SCALE / f64::from(sent_height))
    } else {
        (1.0, 1.0)
    };
    PageLayout {
        lines: lines
            .into_iter()
            .map(|line| LayoutLine {
                bounds: box_of(&line.bbox, scale),
                text: line.text,
                bold: line.bold,
                family: family_of(line.font.as_deref()),
                legible: line.legible,
                kept: None,
            })
            .collect(),
        graphics: graphics
            .into_iter()
            .map(|graphic| LayoutGraphic {
                kind: graphic.kind.unwrap_or_else(|| DEFAULT_GRAPHIC.to_owned()),
                bounds: box_of(&graphic.bbox, scale),
            })
            .collect(),
    }
}

fn box_of(values: &[f64], (scale_x, scale_y): (f64, f64)) -> LayoutBox {
    let (x0, x1) = (in_page(values[0] * scale_x), in_page(values[2] * scale_x));
    let (y0, y1) = (in_page(values[1] * scale_y), in_page(values[3] * scale_y));
    LayoutBox { left: x0.min(x1), top: y0.min(y1), right: x0.max(x1), bottom: y0.max(y1) }
}

fn in_page(value: f64) -> f64 {
    value.clamp(0.0, LayoutBox::SCALE)
}

fn family_of(font: Option<&str>) -> FontFamily {
    match font.map(str::to_ascii_lowercase).as_deref() {
        Some("serif") => FontFamily::Serif,
        Some("mono") => FontFamily::Mono,
        _ => FontFamily::Sans,
    }
}
