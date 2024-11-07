use std::cmp::max;
use euclid::default::{Point2D, Vector2D};
use crate::more_itertools::MoreIterTools;
use crate::geometry::{Orthopolygonlike, Polygon};

pub fn to_accurate_polygon<Ortho: Orthopolygonlike>(orthopolygon: &Ortho) -> Polygon<f64> {
    let mut vertices = Vec::new();
    
    for (p0, p1, p2, p3) in orthopolygon.vertices().circular_tuples() {
        // We need 4 points and 3 segments between them
        let (len_prev, dir_prev) = get_len_and_dir(p1 - p0);
        let (len_cur,  dir_cur ) = get_len_and_dir(p2 - p1);
        let (len_next, dir_next) = get_len_and_dir(p3 - p2);
        
        // Both the previous and the next segment lie on the same side of the current
        let is_convex = dir_prev + dir_next == Vector2D::zero();
        // The previous and the current segments form a corner
        let is_corner = len_prev > 1 && len_cur > 1;
        // Very long horizontal or vertical segment
        let is_too_long = len_cur > MAX_SLOPE_RATIO;
        // Segment of length 1 and surrounded by two segments on the same side
        let is_dimple = is_convex && len_cur == 1;
        // Segment of length 1 and surrounded by two long segments on the same side
        let is_pin = is_dimple && len_prev > 1 && len_next > 1;
        
        if is_corner {
            // Add the corner point
            let corner = p1.to_f64() + (dir_cur - dir_prev).to_f64() * CORNER_OFFSET;
            vertices.push(corner);
        }
        
        if is_pin || is_too_long {
            // Too long segments and one pixel wide pins, instead of the center point,
            // get two points at a fixed distance from the ends
            let offset = if is_pin { CORNER_OFFSET } else { MAX_SLOPE_RATIO as f64 * 0.5 };
            let offset_vector = dir_cur.to_f64() * offset;
            vertices.push(p1.to_f64() + offset_vector);
            vertices.push(p2.to_f64() - offset_vector);
        } else {
            let center = p1.to_f64().lerp(p2.to_f64(), 0.5);
            // Most new vertices will be at the segment centers,
            // except dimples that are offset
            if is_dimple {
                let offset_dir = if len_prev > 1 {
                    -dir_cur
                } else if len_next > 1 {
                    dir_cur
                } else {
                    -dir_prev
                };
                vertices.push(center + offset_dir.to_f64() * CORNER_OFFSET);
            } else {
                vertices.push(center);
            }
        }
    }
    
    if vertices.len() == 4 {
        // Single-pixel contour is returned as is
        orthopolygon.to_polygon()
    } else {
        Polygon::from(simplify(vertices))
    }
}

fn get_len_and_dir(v: Vector2D<i32>) -> (i32, Vector2D<i32>) {
    let len = max(v.x.abs(), v.y.abs());
    if len <= 0 {
        println!("Warinig! {v:?}");
    }
    let dir = v / len;
    (len, dir)
}

fn simplify(vertices: Vec<Point2D<f64>>) -> Vec<Point2D<f64>> {
    vertices.iter()
        .circular_tuples()
        .filter_map(|(&p0, &p1, &p2)| {
            let dir_prev = (p1 - p0).normalize();
            let dir_next = (p2 - p1).normalize();
            let collinear = (dir_next - dir_prev).square_length() <= SQUARED_EPSILON;
            if collinear { None } else { Some(p1) }
        })
        .collect()
}

const MAX_SLOPE_RATIO: i32 = 12;
const CORNER_OFFSET: f64 = 0.25;
const SQUARED_EPSILON: f64 = 1e-6;
