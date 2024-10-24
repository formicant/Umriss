use std::iter;
use itertools::Itertools;
use crate::image_contour_collection::{ImageContourCollection, Contour};

pub fn contours_to_svg(contour_collection: &ImageContourCollection) -> String {
    let (width, height) = contour_collection.dimensions();
    let mut paths = contour_collection.non_hole_contours().map(get_path);
    let path_lines = paths.join("\n  ");
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{width}" height="{height}">
 <g fill="blue" fill-opacity="0.5" stroke="blue" stroke-width="0.1">
  {path_lines}
 </g>
</svg>"#)
}

fn get_path(non_hole_contour: Contour) -> String {
    let mut nodes = Vec::new();
    let contours = iter::once(non_hole_contour)
        .chain(non_hole_contour.children());
    for contour in contours {
        add_nodes(&mut nodes, contour);
    }
    let data = nodes.concat();
    format!(r#"<path d="{data}"/>"#)
}

fn add_nodes(nodes: &mut Vec<String>, contour: Contour) {
    let mut first_x = None;
    for point in contour.even_points() {
        let (x, y) = point;
        if let None = first_x {
            first_x = Some(x);
            nodes.push(format!("M{x},{y}"));
        } else {
            nodes.push(format!("H{x}V{y}"));
        }
    }
    nodes.push(format!("H{}Z", first_x.unwrap()));
}