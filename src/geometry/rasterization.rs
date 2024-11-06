use std::collections::BTreeSet;
use itertools::Itertools;
use num_traits::int::PrimInt;
use num_traits::cast::NumCast;
use image::{GrayImage, Luma};
use crate::more_itertools::MoreIterTools;
use super::{Number, Orthopolygonlike};

/// Draws one or more `orthopolygons` in a `canvas` image.
/// 
/// The value of each pixel of the canvas inside an orthopolygon
/// (more precisely, by the even-odd rule)
/// is modified using the `draw_pixel` function.
pub fn draw_orthopolygons<'a, N, Ortho>(
    canvas: &mut GrayImage,
    draw_pixel: impl Fn(u8) -> u8,
    orthopolygons: impl Iterator<Item = &'a Ortho>,
) where
    N: Number + PrimInt,
    Ortho: Orthopolygonlike<N> + 'a,
{
    let edges: Vec<_> = orthopolygons
        .flat_map(|p| p.even_vertices().circular_pairs())
        .map(|(u, v)| if u.y < v.y { (u.y, v.x, v.y) } else { (v.y, v.x, u.y) })
        .sorted()
        .collect();
    
    let mut parity = false;
    let mut edge_index = 0;
    let mut active_edges = BTreeSet::new();
    let Some((mut y, _, _)) = edges.iter().next() else { return };
    
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
        
        let mut prev_x = N::min_value();
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
        y += N::one();
    }
}

fn draw_horizontal_line<N: Number + PrimInt>(
    canvas: &mut GrayImage,
    draw_pixel: impl Fn(u8) -> u8,
    y: N, x0: N, x1: N,
) {
    let y = <u32 as NumCast>::from(y).unwrap();
    let x0 = <u32 as NumCast>::from(x0).unwrap();
    let x1 = <u32 as NumCast>::from(x1).unwrap();
    for x in x0..x1 {
        let Luma([value]) = canvas.get_pixel_mut(x, y);
        *value = draw_pixel(*value);
    }
}
