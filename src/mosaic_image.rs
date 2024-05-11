use image::DynamicImage;

#[path = "./utils.rs"]
mod utils;

use utils::calculate_optimal_image_resolution;

pub struct MosaicImage {
    target: DynamicImage,
    images: Vec<String>,
    used: Vec<String>,
    closest: Vec<String>,
    processed_width: u32,
    processed_height: u32,
    pub grid_resolution: u32,
}

impl MosaicImage {
    pub fn new(target: DynamicImage, images: Vec<String>, grid_resolution: Option<u32>, include: f32) -> Self {
        let mut instance = MosaicImage {
            target,
            images: Vec::new(),
            used: Vec::new(),
            closest: Vec::new(),
            processed_width: 0,
            processed_height: 0,
            grid_resolution: 0,
        };

        // Only include n% of images.
        if include > 0.0 && include <= 1.0 {
            let num_images = (images.len() as f32 * include) as usize;
            instance.images = images.into_iter().take(num_images).collect();
        } else {
            instance.images = images;
        }

        match grid_resolution {
            Some(n) => instance.grid_resolution = n,
            None => instance.grid_resolution = calculate_optimal_image_resolution(instance.target.clone(), instance.images.len() as u32)
        }

        return instance;

    }
}