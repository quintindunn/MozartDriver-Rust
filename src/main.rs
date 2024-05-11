mod mosaic_image;
mod mosaic_image_child;

use mosaic_image::MosaicImage;

use std::fs::read_dir;
use std::path::{Path};
use image::GenericImageView;
use image::imageops::FilterType;

fn main() {
    const INCLUDE: f32 = 1.0;

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

    let (original_width, original_height) = target_image.clone().dimensions();
    let resized_image = target_image.resize(original_width/2, original_height/2, FilterType::Lanczos3);


    // Create new MosaicImage
    let mut mosaic_image = MosaicImage::new(target_image, src_paths, None, INCLUDE);
    println!("Optimal Grid Resolution: {:?}", mosaic_image.grid_resolution);
    println!("Using {}/{} images.", mosaic_image.images.len(), src_image_count);

    println!("{}", MosaicImage::calculate_blocks(&mosaic_image));

    println!("Creating color grids!");
    MosaicImage::create_color_grids(&mut mosaic_image);

    println!("Compiling closest blocks!");
    MosaicImage::compile_closest_images(&mut mosaic_image);

    println!("Overlaying blocks!");
    let output = MosaicImage::overlay_blocks(&mut mosaic_image);

    output.save("output.png").expect("Failed to save image!");
}