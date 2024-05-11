use image::{DynamicImage, GenericImageView};

pub fn calculate_optimal_image_resolution(target: DynamicImage, image_count: u32) -> u32 {
    /*
    Calculates the optimal inverse resolution to include images.

    # Arguments

    * `target` - Target image

    * `image_count` - Number of images used to build final image.

    # Returns
    * The calculated optimal inverse resolution.
     */

    let (target_width, target_height) = target.dimensions();

    let total_area: u32 = target_width * target_height;
    let chunk_area: f64 = (total_area / image_count) as f64;

    return chunk_area.sqrt().ceil() as u32;
}
