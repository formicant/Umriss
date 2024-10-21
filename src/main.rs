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
        if cs.contour_points.len() != contours.contour_points.len() {
            panic!();
        }
    }
    let time = start.elapsed();
    
    let svg = get_svg(&contours, &image_file.strip_prefix("img/").unwrap());
    fs::write("img/out.svg", svg)?;
    
    for (i, p) in contours.contour_points.iter().enumerate() {
        println!("{i:5}: {p:?}");
    }
    println!("Elapsed: {}", time.as_micros() / iterations);
    
    Ok(())
}

fn get_svg(contours: &ImageContours, image_file: &str) -> String {
    let (width, height) = contours.dimensions();
    let points = &contours.contour_points[..];
    
    let mut nodes = Vec::new();
    let mut visited = vec![false; points.len()];
    let mut index = 1;
    while index > 0 {
        let mut first = None;
        loop {
            let p = &points[index];
            first = match first {
                None => {
                    nodes.push(format!("M {} {} ", p.x, p. y));
                    Some(index)
                },
                Some(f) if f == index  => {
                    nodes.push(format!("H {} Z ", p.x));
                    None
                },
                _ => {
                    nodes.push(format!("H {} V {} ", p.x, p.y));
                    first
                },
            };
            if first == None { break; }
            visited[index] = true;
            index = p.next;
        }
        
        loop {
            index += 1;
            if index >= points.len() {
                index = 0;
                break;
            }
            if !visited[index] {
                break;
            }
        }
    }
    
    let path = nodes.concat();
    
    return format!(r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{width}" height="{height}">
  <image href="{image_file}" image-rendering="pixelated" opacity="0.25" />
  <path fill="none" stroke="blue" stroke-width="0.1" d="{path}" />
</svg>"#);
}