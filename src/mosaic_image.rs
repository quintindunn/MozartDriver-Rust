use image::{DynamicImage, GenericImageView};
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
    closest: Vec<MosaicImageChild>,
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

        let resized_width = (width as f32 / instance.grid_resolution as f32).ceil() as u32 * instance.grid_resolution;
        let resized_height = (height as f32 / instance.grid_resolution as f32).ceil() as u32 * instance.grid_resolution;

        return (resized_width / instance.grid_resolution) * (resized_height / instance.grid_resolution);
    }

    pub fn create_color_grids(instance: &mut MosaicImage) {
        let grid_resolution = instance.grid_resolution;

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

    fn find_closest_child<'a>(images: &Vec<MosaicImageChild>, rgb: &(u8, u8, u8)) -> Option<MosaicImageChild> {
        // Using Euclidean distance calculate to calculate which image in `instance.images`
        // has the closest average color to the target block.
        fn color_distance(c1: &(u8, u8, u8), c2: &(u8, u8, u8)) -> f64 {
            let (r1, g1, b1): (u8, u8, u8) = *c1;
            let (r2, g2, b2): (u8, u8, u8) = *c2;

            return ((r1 as f64 - r2 as f64).powi(2) + (g1 as f64 - g2 as f64).powi(2) + (b1 as f64 - b2 as f64).powi(2)).sqrt()
        }

        // Iterate through all values, store the closest value and index of said value
        let mut closest_distance: f64 = 99999.9;
        let mut closest_idx = 0;

        for (k, image) in images.iter().enumerate() {
            let distance = color_distance(&image.avg_rgb_values, &rgb);

            if distance < closest_distance {
                closest_distance = distance;
                closest_idx = k;
            }
        }
        if closest_idx < images.len() {
            return Some(images[closest_idx].clone());
        }
        return None

    }
    pub fn compile_closest_images(instance: &mut MosaicImage) {
        {
            for (k, target) in instance.rgb_targets.iter().enumerate() {
                let found = MosaicImage::find_closest_child(&instance.images, target);

                match found {
                    Some(image) => {
                        instance.images.remove(0);
                        instance.closest.push(image.clone());
                    }
                    None => {}
                }


                if k % 20 == 0 {
                    println!("Compiling images {}%!", (k as f32 / instance.rgb_targets.len() as f32 * 100f32) as u32);
                }
            }
            println!("Compiling images {}%!", 100);
        }
    }
}