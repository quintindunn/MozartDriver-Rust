mod mosaic_image;

use std::fs::read_dir;
use std::path::{Path};
use mosaic_image::MosaicImage;

fn main() {
    // Get the images to build the mozart
    let mut target_images = read_dir("./test_images/targets").unwrap();
    let src_images = read_dir("./test_images/src").unwrap();

    let src_names: Vec<String> = src_images
        .filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().ok()).flatten())
        .collect();

    let target_binding = target_images.next().unwrap().unwrap().path();
    let target = target_binding.to_str().unwrap();

    let src_image_count: usize = src_names.len();


    println!("Target image: {:?}", target);
    println!("Total src images: {src_image_count}");

    // Load the target image
    let target_path = Path::new(target);
    let target_image = image::open(target_path).unwrap();


    // Create new MosaicImage
    let mosaic_image = MosaicImage::new(target_image.clone(), src_names, None, 1.0);
    println!("Optimal Grid Resolution: {:?}", mosaic_image.grid_resolution)

}