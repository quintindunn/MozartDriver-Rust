use std::fs::File;
use image::{DynamicImage, GenericImageView, ImageFormat};
use image::imageops::FilterType;

#[path = "./utils.rs"]
mod utils;

#[path = "./mosaic_image_child.rs"]
mod mosaic_image_child;

use utils::calculate_optimal_image_resolution;
use mosaic_image_child::MosaicImageChild;

use rayon::prelude::*;


pub struct MosaicImage {
    target: DynamicImage,
    image_paths: Vec<String>,
    images: Vec<MosaicImageChild>,
    used: Vec<String>,
    closest: Vec<String>,
    processed_width: u32,
    processed_height: u32,
    target_width: u32,
    target_height: u32,
    pub grid_resolution: u32,
    rgb_targets: Vec<(u8, u8, u8)>
}

impl MosaicImage {
    pub fn new(target: DynamicImage, images: Vec<String>, grid_resolution: Option<u32>, include: f32) -> Self {
        let mut instance = MosaicImage {
            target,
            image_paths: Vec::new(),
            images: Vec::new(),
            used: Vec::new(),
            closest: Vec::new(),
            processed_width: 0,
            processed_height: 0,
            grid_resolution: 0,
            target_width: 0,
            target_height: 0,
            rgb_targets: Vec::new()
        };

        // Only include n% of images.
        if include > 0.0 && include <= 1.0 {
            let num_images = (images.len() as f32 * include) as usize;
            instance.image_paths = images.clone().into_iter().take(num_images).collect();
        } else {
            instance.image_paths = images.clone();
        }

        // Setup grid resolution
        match grid_resolution {
            Some(n) => instance.grid_resolution = n,
            None => instance.grid_resolution = calculate_optimal_image_resolution(instance.target.clone(), instance.image_paths.len() as u32)
        }

        // Load the src images:
        let new_images: Vec<MosaicImageChild> = instance.image_paths
            .par_iter()
            .map(|image_path| {
                let image = image::open(image_path).unwrap();
                MosaicImageChild::new(image)
            })
            .collect();

        instance.images.extend(new_images);

        return instance;
    }

    fn calculate_blocks(instance: &MosaicImage) -> u32 {
        /*
        Calculates the amount of blocks that will be present in instance.rgb_targets for
        about an inadequate amount of blocks.

        # Returns
        * u32 of estimated number of blocks
         */

        let (width, height) = instance.target.clone().dimensions();

        let resized_width = (width / instance.grid_resolution) * instance.grid_resolution;
        let resized_height = (width / instance.grid_resolution) * instance.grid_resolution;

        return (resized_width / instance.grid_resolution) * (resized_height / instance.grid_resolution);
    }

    pub fn create_color_grids(instance: &mut MosaicImage) {
        let grid_resolution = instance.grid_resolution;

        // Make sure we have enough images:
        let block_count: u32 = MosaicImage::calculate_blocks(instance);
        if block_count > instance.images.len() as u32 {
            panic!("Not enough src images to fulfill the image!");
        }

        let (target_width, target_height): (u32, u32) = instance.target.clone().dimensions();

        // Calculate & resize the target's dimensions
        let width_resized = (target_width as f64 / grid_resolution as f64).ceil() as u32 * grid_resolution;
        let height_resized = (target_height as f64 / grid_resolution as f64).ceil() as u32 * grid_resolution;

        println!("Resizing to {width_resized}x{height_resized}");
        instance.target = instance.target.resize_exact(width_resized, height_resized, FilterType::Lanczos3);

        // Create blocks to overlay onto final image:
        let mut blocks: Vec<MosaicImageChild> = Vec::new();


        for j in (0..height_resized).step_by(grid_resolution as usize) {
            for i in (0..width_resized).step_by(grid_resolution as usize) {
                let cropped = instance.target.crop(i, j, i + grid_resolution, j + grid_resolution);
                let child = MosaicImageChild::new(cropped);
                blocks.push(child);
            }
        }

        // Store the block's average RGB values for final stitching.
        for block in blocks {
            instance.rgb_targets.push(block.avg_rgb_values);
        }
    }
}