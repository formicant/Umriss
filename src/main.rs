mod image_contour_collection;
mod geometry;
mod test_images;
mod silly_svg;
mod glyph;
mod book;

use std::fs;
use std::time::Instant;
use book::Book;
use image_contour_collection::ImageContourCollection;
use silly_svg::contours_to_svg;
use test_images::{get_test_images, get_test_image};
use euclid::default::{Point2D, Size2D, Vector2D};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    // process_test_images();
    // measure_performance("noise_200x100_white", true, 1000);
    // measure_performance("text_5012x7060_math", true, 100);
    // measure_performance("text_7717x10672_gospel", true, 50);
    
    let images = ["ku/003", "ku/021"];
    let collections = images.iter().map(|name| ImageContourCollection::black_on_white(&get_test_image(name)));
    let book = Book::new(collections);
    
    println!("{:?}", book);
    
    Ok(())
}

fn process_test_images() {
    fs::create_dir_all("output").unwrap();
    println!("Processing test images:");
    let inverted = true;
    for (name, image) in get_test_images() {
        println!("- {name}");
        let contour_collection = ImageContourCollection::new(&image, inverted);
        let svg_contents = contours_to_svg(&contour_collection);
        fs::write(format!("output/{name}.svg"), svg_contents).unwrap();
    }
    println!("");
}

fn measure_performance(name: &str, inverted: bool, iterations: usize) {
    let image = get_test_image(name);
    println!("Measuring performance on '{name}'...");
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = ImageContourCollection::new(&image, inverted);
    }
    let time = start.elapsed();
    
    let per_iteration = time.as_secs_f64() * 1000.0 / iterations as f64;
    println!("{per_iteration:.3} ms");
}