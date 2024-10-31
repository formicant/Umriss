use crate::image_contour_collection::Contour;

/// Position of a point relative to a contour.
pub enum PointPosition {
    Outside,
    Vertex,
    Edge,
    Inside,
}

/// Uses ray casting algorithm to determine if the point is
/// inside the contour, outside, on an edge, or at a vertex.
pub fn get_point_position_relative_to_contour(point: (u32, u32), contour: Contour) -> PointPosition {
    let (x, y) = point;
    let mut intersections = 0;
    
    for ((x0, y0), (x1, y1)) in contour.even_point_pairs() {
        let is_x_between = (x0 <= x && x < x1) || (x1 <= x && x < x0);
        if y0 < y && is_x_between {
            intersections += 1;
        }
        if (y0 == y && (x == x0 || x == x1)) || (y == y1 && x == x1) {
            return PointPosition::Vertex;
        }
        let is_y_between = (y0 <= y && y < y1) || (y1 <= y && y < y0);
        if  (y == y0 && is_x_between) || (x == x1 && is_y_between) {
            return PointPosition::Edge;
        }
    }
    if intersections % 2 == 0 { PointPosition::Outside } else { PointPosition::Inside }
}
