mod image_contour_collection;

use std::{fs, iter};
use std::error::Error;
use std::time::Instant;
use image::ImageReader;
use image_contour_collection::ImageContourCollection;

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let image_file = "img/test.png";
    // let image_file = "img/page.png";
    let image = ImageReader::open(image_file)?.decode()?.into_luma8();
    let contours = ImageContourCollection::new(&image);
    
    let iterations = 250;
    let start = Instant::now();
    for _ in 0..iterations {
        let cs = ImageContourCollection::new(&image);
        if cs.point_list.len() != contours.point_list.len() {
            panic!();
        }
    }
    let time = start.elapsed();
    
    let svg = get_svg(&contours, &image_file.strip_prefix("img/").unwrap());
    fs::write("img/out.svg", svg)?;
    
    for (i, p) in contours.point_list.iter().enumerate() {
        println!("{i:5}: {p:?}");
    }
    println!("Elapsed: {}", time.as_micros() / iterations);
    
    Ok(())
}

fn get_svg(contours: &ImageContourCollection, image_file: &str) -> String {
    let (width, height) = contours.dimensions();
    let mut paths = Vec::new();
    
    for outer_contour in contours.non_hole_contours() {
        let mut nodes = Vec::new();
        
        for contour in iter::once(outer_contour).chain(outer_contour.children()) {
            let control_points: Vec<_> = contour.even_points().collect();
            let (x0, y0) = control_points[0];
            nodes.push(format!("M {} {} ", x0, y0));
            for (x, y) in control_points[1..].iter() {
                nodes.push(format!("H {} V {} ", x, y));
            }
            nodes.push(format!("H {} Z ", x0));
        }
        paths.push(format!(r#"    <path d="{}" />
"#, nodes.concat()));
    }
    
    return format!(r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{width}" height="{height}">
  <image href="{image_file}" image-rendering="pixelated" opacity="0.1" />
  <g stroke="blue" fill="blue" fill-opacity="0.5" stroke-width="0.1">
{}
  </g>
</svg>"#, paths.concat());
}
