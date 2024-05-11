use image::{DynamicImage, GenericImageView};

#[derive(Clone)]
pub struct MosaicImageChild {
    image: DynamicImage,
    pub avg_rgb_values: (u8, u8, u8)
}

impl MosaicImageChild {
    pub fn new(src_image: DynamicImage) -> MosaicImageChild {
        let mut instance = MosaicImageChild {
            image: src_image,
            avg_rgb_values: (0u8, 0u8, 0u8)
        };

        let (mut ar , mut ag, mut ab): (u64, u64, u64) = (0, 0, 0);
        let mut k: u64 = 0;

        for pixel in instance.image.clone().pixels() {
            let rgb = pixel.2;
            ar += rgb[0] as u64;
            ag += rgb[1] as u64;
            ab += rgb[2] as u64;

            k += 1;
        }

        let avg_r: u8 = (ar / k) as u8;
        let avg_g: u8 = (ag / k) as u8;
        let avg_b: u8 = (ab / k) as u8;
        instance.avg_rgb_values = (avg_r, avg_g, avg_b);

        return instance;
    }
}