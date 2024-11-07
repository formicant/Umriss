mod image_contour_collection;
mod geometry;
mod test_images;
mod silly_svg;
mod glyph;
mod book;
mod more_itertools;
mod approximation;

use std::fs;
use std::time::Instant;
use book::Book;
use image_contour_collection::ImageContourCollection;
use silly_svg::{write_contour_collection_as_svg_file, write_book_as_multiple_svg_files};
use test_images::{get_test_images, get_test_image};
use approximation::to_accurate_polygon;

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    process_test_images();
    // measure_performance("noise_200x100_white", true, 1000);
    // measure_performance("text_5012x7060_math", true, 100);
    // measure_performance("text_7717x10672_gospel", true, 50);
    
    // process_ku();
    
    Ok(())
}

fn process_ku() {
    let start_decoding = Instant::now();
    let images: Vec<_> = (1..=402).map(|i| get_test_image(&format!("ku/{i:03}"))).collect();
    let decoding = start_decoding.elapsed();
    println!("Decoding:   {:.3} s", decoding.as_secs_f64());
    
    let start_contouring = Instant::now();
    let contours: Vec<_> = images.iter().map(ImageContourCollection::black_on_white).collect();
    let contouring = start_contouring.elapsed();
    println!("Contouring: {:.3} s", contouring.as_secs_f64());
    
    let start_booking = Instant::now();
    let book = Book::new(contours.into_iter());
    let booking = start_booking.elapsed();
    println!("Booking:    {:.3} s", booking.as_secs_f64());
    
    let start_writing = Instant::now();
    write_book_as_multiple_svg_files(&book);
    let writing = start_writing.elapsed();
    println!("Writing:    {:.3} s", writing.as_secs_f64());
}

fn process_test_images() {
    fs::create_dir_all("output").unwrap();
    println!("Processing test images:");
    let inverted = true;
    for (name, image) in get_test_images() {
        println!("- {name}");
        let contour_collection = ImageContourCollection::new(&image, inverted);
        let approximation: Vec<_> = contour_collection.all_contours().map(|c| to_accurate_polygon(&c)).collect();
        write_contour_collection_as_svg_file(&contour_collection, approximation, &name);
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