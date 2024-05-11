mod mosaic_image;
mod mosaic_image_child;

use mosaic_image::MosaicImage;

use std::fs::read_dir;
use std::path::{Path};

fn main() {
    // Get the images to build the mozart
    let mut target_images = read_dir("./test_images/targets").unwrap();
    let src_images = read_dir("./test_images/src").unwrap();

    let src_paths: Vec<String> = src_images
        .filter_map(|entry| entry.ok().map(|e| e.path().into_os_string().into_string().ok()).flatten())
        .collect();

    let target_binding = target_images.next().unwrap().unwrap().path();
    let target = target_binding.to_str().unwrap();

    let src_image_count: usize = src_paths.len();


    println!("Target image: {:?}", target);
    println!("Total src images: {src_image_count}");

    // Load the target image
    let target_path = Path::new(target);
    let target_image = image::open(target_path).unwrap();


    // Create new MosaicImage
    let mut mosaic_image = MosaicImage::new(target_image.clone(), src_paths, None, 0.016);
    println!("Optimal Grid Resolution: {:?}", mosaic_image.grid_resolution);

    println!("Creating color grids!");
    MosaicImage::create_color_grids(&mut mosaic_image);

    println!("Compiling closest blocks!");
    MosaicImage::compile_closest_images(&mut mosaic_image);
}