use std::collections::BTreeSet;
use itertools::Itertools;
use image::{GrayImage, Luma};
use crate::more_itertools::MoreIterTools;
use super::Orthopolygonlike;

/// Draws one or more `orthopolygons` in a `canvas` image.
/// 
/// The value of each pixel of the canvas inside an orthopolygon
/// (more precisely, by the even-odd rule)
/// is modified using the `draw_pixel` function.
pub fn draw_orthopolygons<'a, Ortho>(
    canvas: &mut GrayImage,
    draw_pixel: impl Fn(u8) -> u8,
    orthopolygons: impl Iterator<Item = &'a Ortho>,
) where Ortho: Orthopolygonlike + 'a {
    let edges: Vec<_> = orthopolygons
        .flat_map(|p| p.even_vertices().circular_pairs())
        .map(|(u, v)| if u.y < v.y { (u.y, v.x, v.y) } else { (v.y, v.x, u.y) })
        .sorted()
        .collect();
    
    let mut parity = false;
    let mut edge_index = 0;
    let mut active_edges = BTreeSet::new();
    let Some(&(mut y, _, _)) = edges.iter().next() else { return };
    
    while edge_index < edges.len() || !active_edges.is_empty() {
        debug_assert!(!parity);
        
        while edge_index < edges.len() {
            let (y0, x, y1) = edges[edge_index];
            if y0 == y {
                active_edges.insert((x, y1));
                edge_index += 1;
            } else {
                break;
            }
        }
        
        let mut prev_x = i32::MIN;
        active_edges.retain(|&(x, y1)|
            if y1 > y {
                if parity {
                    draw_horizontal_line(canvas, &draw_pixel, y, prev_x, x);
                }
                parity = !parity;
                prev_x = x;
                true
            } else {
                false
            }
        );
        y += 1;
    }
}

fn draw_horizontal_line(
    canvas: &mut GrayImage,
    draw_pixel: impl Fn(u8) -> u8,
    y: i32, x0: i32, x1: i32,
) {
    for x in x0..x1 {
        let Luma([value]) = canvas.get_pixel_mut(x as u32, y as u32);
        *value = draw_pixel(*value);
    }
}
