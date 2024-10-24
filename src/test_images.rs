use std::{fs, path::{Path, PathBuf}};
use image::{GrayImage, ImageReader};

pub fn get_test_images() -> impl Iterator<Item = (String, GrayImage)> {
    let files = fs::read_dir(TEST_IMAGES_DIRECTORY).unwrap();
    files.map(|file| {
        let path = file.unwrap().path();
        let name: String = path.file_stem().unwrap_or(path.file_name().unwrap()).to_os_string().into_string().unwrap();
        let image = load_image(path);
        (name, image)
    })
}

pub fn get_test_image(name: &str) -> GrayImage {
    let path = Path::new(TEST_IMAGES_DIRECTORY).join(format!("{name}.png"));
    load_image(path)
}

fn load_image(path: PathBuf) -> GrayImage {
    let image = ImageReader::open(path).unwrap().decode().unwrap();
    image.into_luma8()
}

const TEST_IMAGES_DIRECTORY: &'static str = "test_images";
