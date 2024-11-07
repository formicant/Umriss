use euclid::{default::{Point2D, Vector2D}, vec2};
use crate::more_itertools::MoreIterTools;
use crate::geometry::{Orthopolygonlike, Polygon};

pub fn to_accurate_polygon<Ortho: Orthopolygonlike>(orthopolygon: &Ortho) -> Polygon<f64> {
    let mut vertices = Vec::new();
    
    let edges = orthopolygon.edges().map(Edge::new);
    for (prev, cur, next) in edges.circular_tuples() {
        let is_convex = prev.direction + next.direction == Vector2D::zero();
        let is_pimple = is_convex && !cur.is_long;
        let is_pin = is_pimple && prev.is_long && next.is_long;
        let starts_at_corner = prev.is_long && cur.is_long;
        
        if starts_at_corner {
            // Add the corner point
            let offset_vector = (cur.direction - prev.direction).to_f64() * CORNER_OFFSET;
            vertices.push(cur.start + offset_vector);
        }
        
        if is_pin || cur.is_too_long {
            // Too long segments and one pixel wide pins, instead of the center point,
            // get two points at a fixed distance from the ends
            let offset = if is_pin { CORNER_OFFSET } else { MAX_SLOPE_RATIO as f64 * 0.5 };
            let offset_vector = cur.direction.to_f64() * offset;
            vertices.push(cur.start + offset_vector);
            vertices.push(cur.end - offset_vector);
        } else {
            // Most new vertices will be at the segment centers,
            // except pimples that are offset
            let center = cur.start.lerp(cur.end, 0.5);
            if is_pimple {
                let offset_direction = if prev.is_long {
                    -cur.direction
                } else if next.is_long{
                    cur.direction
                } else {
                    -prev.direction
                };
                let offset_vector = offset_direction.to_f64() * CORNER_OFFSET;
                vertices.push(center + offset_vector);
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

/// Temporarily stores an edge with some calculated values.
#[derive(Debug, Clone, Copy)]
struct Edge {
    start: Point2D<f64>,
    end: Point2D<f64>,
    direction: Vector2D<i32>,
    is_long: bool,
    is_too_long: bool,
}

impl Edge {
    pub fn new(points: (Point2D<i32>, Point2D<i32>)) -> Self {
        let (start, end) = points;
        let vector = end - start;
        let direction = vec2(vector.x.signum(), vector.y.signum());
        let length = i32::max(vector.x.abs(), vector.y.abs());
        let is_long = length > 1;
        let is_too_long = length > MAX_SLOPE_RATIO;
        Self { start: start.to_f64(), end: end.to_f64(), direction, is_long, is_too_long }
    }
}

fn simplify(vertices: Vec<Point2D<f64>>) -> Vec<Point2D<f64>> {
    vertices.iter()
        .circular_tuples()
        .filter_map(|(&p0, &p1, &p2)| {
            let d0 = p0.x * (p1.y - p2.y);
            let d1 = p1.x * (p2.y - p0.y);
            let d2 = p2.x * (p0.y - p1.y);
            let collinear = (d0 + d1 + d2).abs() <= EPSILON;
            if collinear { None } else { Some(p1) }
        })
        .collect()
}

const MAX_SLOPE_RATIO: i32 = 12;
const CORNER_OFFSET: f64 = 0.25;
const EPSILON: f64 = 1e-3;
