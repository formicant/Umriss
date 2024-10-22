mod image_contours;

use std::fs;
use std::error::Error;
use std::time::Instant;
use image::ImageReader;
use image_contours::ImageContours;

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let image_file = "img/test.png";
    // let image_file = "img/bull.png";
    let image = ImageReader::open(image_file)?.decode()?.into_luma8();
    let contours = ImageContours::new(&image);
    
    let iterations = 250;
    let start = Instant::now();
    for _ in 0..iterations {
        let cs = ImageContours::new(&image);
        if cs.table.len() != contours.table.len() {
            panic!();
        }
    }
    let time = start.elapsed();
    
    let svg = get_svg(&contours, &image_file.strip_prefix("img/").unwrap());
    fs::write("img/out.svg", svg)?;
    
    for (i, p) in contours.table.iter().enumerate() {
        println!("{i:5}: {p:?}");
    }
    println!("Elapsed: {}", time.as_micros() / iterations);
    
    Ok(())
}

fn get_svg(contours: &ImageContours, image_file: &str) -> String {
    let (width, height) = contours.dimensions();
    let mut paths = Vec::new();
    
    for contour in contours.outermost_contours() {
        let control_points: Vec<_> = contour.control_points().collect();
        let (x0, y0) = control_points[0];
        let mut nodes = vec![format!("M {} {} ", x0, y0)];
        for (x, y) in control_points[1..].iter() {
            nodes.push(format!("H {} V {} ", x, y));
        }
        nodes.push(format!("H {} Z", x0));
        paths.push(format!(r#"  <path fill="none" stroke="blue" stroke-width="0.1" d="{}" />
"#, nodes.concat()));
    }
    
    return format!(r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{width}" height="{height}">
  <image href="{image_file}" image-rendering="pixelated" opacity="0.25" />
{}</svg>"#, paths.concat());
}
