use std::{fs, path::{Path, PathBuf}};
use image::{GrayImage, ImageReader};
use itertools::Itertools;

pub fn get_test_images() -> impl Iterator<Item = (String, GrayImage)> {
    let files: Vec<_> = fs::read_dir(TEST_IMAGES_DIRECTORY).unwrap()
        .map(|file| file.unwrap())
        .filter(|file| file.file_type().unwrap().is_file())
        .map(|file| {
            let path = file.path();
            let name: String = path.file_stem().unwrap_or(path.file_name().unwrap()).to_os_string().into_string().unwrap();
            (path, name)
        })
        .sorted_by_key(|(_, name)| name.clone())
        .collect();
    
    let use_whitelist = files.iter().any(|(_, name)| is_in_whitelist(name));
    let condition = if use_whitelist { is_in_whitelist } else { is_not_in_blacklist };
    
    files.into_iter()
        .filter(move |(_, name)| condition(name))
        .map(|(path, name)| {
            let image = load_image(&path);
            (name.clone(), image)
        })
}

pub fn get_test_image(name: &str) -> GrayImage {
    let path = Path::new(TEST_IMAGES_DIRECTORY).join(format!("{name}.png"));
    load_image(&path)
}

fn load_image(path: &PathBuf) -> GrayImage {
    let image = ImageReader::open(path).unwrap().decode().unwrap();
    image.into_luma8()
}

fn is_in_whitelist(name: &str) -> bool {
    name.starts_with("+")
}

fn is_not_in_blacklist(name: &str) -> bool {
    !name.starts_with("-")
}

const TEST_IMAGES_DIRECTORY: &'static str = "test_images";
