use std::collections::HashMap;

use super::mask::Mask;
use crate::rebuild::rect::Rect;

const NONE: usize = usize::MAX;

/// The connected components of a mask in the 8-neighbourhood, found with union-find instead of
/// recursion. [`ids`] holds the component of every pixel, or `None`; [`counts`] and [`boxes`] are per
/// component, numbered in the order their first pixel appears.
pub(crate) struct Labels {
    pub ids: Vec<Option<u32>>,
    pub counts: Vec<usize>,
    pub boxes: Vec<Rect>,
}

impl Labels {
    pub fn of(mask: &Mask) -> Labels {
        let mut parent = vec![NONE; mask.bits.len()];
        link(mask, &mut parent);
        collect(mask, &mut parent)
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }
}

fn link(mask: &Mask, parent: &mut [usize]) {
    let width = mask.width;
    for y in 0..mask.height {
        for x in 0..width {
            if !mask.get(x, y) {
                continue;
            }
            // Joins a set pixel with its set neighbours to the left and in the row above.
            let i = y * width + x;
            parent[i] = i;
            if x > 0 && mask.get(x - 1, y) {
                union(parent, i, i - 1);
            }
            if y == 0 {
                continue;
            }
            if mask.get(x, y - 1) {
                union(parent, i, i - width);
            }
            if x > 0 && mask.get(x - 1, y - 1) {
                union(parent, i, i - width - 1);
            }
            if x < width - 1 && mask.get(x + 1, y - 1) {
                union(parent, i, i - width + 1);
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // coordinates of a page image fit i32
fn collect(mask: &Mask, parent: &mut [usize]) -> Labels {
    let mut ids = vec![None; parent.len()];
    let mut by_root: HashMap<usize, u32> = HashMap::new();
    let mut counts = Vec::new();
    let mut corners: Vec<[usize; 4]> = Vec::new();
    for i in 0..parent.len() {
        if parent[i] == NONE {
            continue;
        }
        let (x, y) = (i % mask.width, i / mask.width);
        let root = find(parent, i);
        let id = *by_root.entry(root).or_insert_with(|| {
            counts.push(0);
            corners.push([x, y, x, y]);
            u32::try_from(counts.len() - 1).unwrap_or(u32::MAX)
        });
        let index = id as usize;
        counts[index] += 1;
        let corner = &mut corners[index];
        corner[0] = corner[0].min(x);
        corner[2] = corner[2].max(x);
        corner[3] = y;
        ids[i] = Some(id);
    }
    let boxes = corners
        .iter()
        .map(|&[left, top, right, bottom]| {
            Rect::new(left as i32, top as i32, (right - left + 1) as i32, (bottom - top + 1) as i32)
        })
        .collect();
    Labels { ids, counts, boxes }
}

fn find(parent: &mut [usize], i: usize) -> usize {
    let mut root = i;
    while parent[root] != root {
        root = parent[root];
    }
    let mut node = i;
    while parent[node] != root {
        let next = parent[node];
        parent[node] = root;
        node = next;
    }
    root
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let root_a = find(parent, a);
    let root_b = find(parent, b);
    if root_a != root_b {
        parent[root_a.max(root_b)] = root_a.min(root_b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal_pixels_join_and_separate_blobs_stay_apart() {
        let mut mask = Mask::new(6, 4);
        for (x, y) in [(0, 0), (1, 1), (2, 2), (5, 0), (5, 1), (3, 0)] {
            mask.set(x, y, true);
        }

        let labels = Labels::of(&mask);

        assert_eq!(labels.len(), 3);
        assert_eq!(labels.counts, vec![3, 1, 2]);
        assert_eq!(labels.boxes[0], Rect::new(0, 0, 3, 3));
        assert_eq!(labels.boxes[2], Rect::new(5, 0, 1, 2));
    }
}
