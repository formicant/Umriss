mod image_contours;

use std::error::Error;
use std::time::Instant;
use image::ImageReader;
use image_contours::ImageContours;

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let image_file = "img/test.png";
    // let image_file = "img/bull.png";
    let image = ImageReader::open(image_file)?.decode()?.into_luma8();
    
    let start = Instant::now();
    let contours = ImageContours::new(&image, |pixel| pixel[0] >= 128);
    let time = start.elapsed();
    
    for (i, p) in contours.contour_points.iter().enumerate() {
        println!("{i:5}: {p:?}");
    }
    println!("Elapsed: {}", time.as_micros());
    
    Ok(())
}
