import logging
import math
import pathlib
import pickle
from typing import Union

from .MosaicImageChild import MosaicImageChild, blur_im
from PIL import Image

logger = logging.getLogger(__name__)


# Implemented!
def calculate_optimal_inverse_resolution(target: Image, include: int) -> int:
    """
    Calculates the optimal inverse resolution to `include` images
    :param target: Target image.
    :param include: Number of images to include.
    :return:
    """
    width, height = target.size

    total_area = width * height

    chunk_area = total_area / include
    return round(math.sqrt(chunk_area))


class MosaicImage:
    def __init__(self,
                 target: Image,
                 images: Union[list[str], list[MosaicImageChild]],
                 inverse_resolution: Union[int, None],
                 include: Union[int, float] = 0.5
                 ):
        """
        :param inverse_resolution: Resolution of image, called "inverse_resolution" because a lower value is better
        :param target: Target image to recreate
        :param images: List of either paths to images OR List of MosaicImageChild images OR some combination of the two.
        :param include: Integer of how many of the images to include in the final image, or pass in a float > 0 <= 1 for
        the percentage of images to include.
        """
        self.target = target

        self.images = []
        self.used = []
        self.closest = []
        self.processed_width = None
        self.processed_height = None

        if inverse_resolution is None:
            if isinstance(include, int):
                self.inverse_resolution = calculate_optimal_inverse_resolution(target, include)
            elif isinstance(include, float):
                self.inverse_resolution = calculate_optimal_inverse_resolution(target, int(len(images) * include))
            else:
                raise TypeError(f"include of type {type(include)} is not in [int, float].")
        else:
            self.inverse_resolution = inverse_resolution

        self.rgb_targets = []

        for im in images:
            if isinstance(im, str):
                logger.debug(f"Loading img {im}")
                im = MosaicImageChild(im, self.inverse_resolution)
            elif isinstance(im, MosaicImageChild):
                if im.resolution != self.inverse_resolution:
                    im.im.resize((self.inverse_resolution, self.inverse_resolution))
            else:
                raise ValueError(f"Image {im} has type {type(im)}, accepted types: list[str], list[MosaicImageChild]")
            self.images.append(im)

    def create_color_grids(self) -> None:
        logger.debug("Creating color grids.")
        """
        Break the target image into a list of MosaicImageChild objects, resized target to fit the amount of blocks
        the block count is dependent on self.inverse_resolution
        :return: None
        """
        block_count = self.calculate_blocks()
        if block_count > len(self.images):
            logging.warning("Not enough chunks.")

        # Break image into blocks of size self.inverse_resolution
        width, height = self.target.size

        # * resize target
        width_t = math.ceil(width / self.inverse_resolution) * self.inverse_resolution
        height_t = math.ceil(height / self.inverse_resolution) * self.inverse_resolution

        self.processed_width = width_t
        self.processed_height = height_t

        self.target = self.target.resize((width_t, height_t))

        blocks = []

        for j in range(0, height, self.inverse_resolution):
            for i in range(0, width, self.inverse_resolution):
                box = (i, j, i + self.inverse_resolution, j + self.inverse_resolution)
                block = MosaicImageChild(self.target.crop(box), self.inverse_resolution)
                blocks.append(block)

        self.rgb_targets = [i.avg_rgb for i in blocks]

    def find_closest_child(self, rgb: tuple[int, int, int]) -> Union[MosaicImageChild, None]:
        logger.debug(f"Finding closest child to {rgb}.")
        """
        Using euclidian distance calculate to calculate which image in `self.images` has the closest average color to
        the target block.
        :param rgb: block's average RGB value
        :return: MosaicImageChild or None, None if we're out of images.
        """
        def color_distance(c1: tuple[int, int, int], c2: tuple[int, int, int]) -> float:
            (r1, g1, b1) = c1
            (r2, g2, b2) = c2
            return math.sqrt((r1 - r2) ** 2 + (g1 - g2) ** 2 + (b1 - b2) ** 2)

        # Find the color in the list that is closest to the target color
        closest_colors = sorted(self.images, key=lambda img: color_distance(img.avg_rgb, rgb))
        if closest_colors:
            return closest_colors[0]
        else:
            return None

    def compile_closest_images(self) -> None:
        logger.debug("Compiling closest images.")
        """
        Loop through the images and map each block to an image in self.images.
        Outputs result to `self.closest`, this is done in order row by row. Remap the locations using the known image
        dimensions.
        :return:
        """
        # TODO: Add multithreading, most likely using concurrent.futures.ThreadPoolExecutor
        for k, target in enumerate(self.rgb_targets):
            found = self.find_closest_child(target)
            if found is None:
                logging.warning("Ran out of blocks!")
                break  # Ran out of blocks.
            self.used.append(found)
            self.images.remove(found)
            self.closest.append(found)
            if k % 20 == 0:
                logger.debug(f"Compiling images {int(k/len(self.rgb_targets)*100)}%.")

    def calculate_blocks(self) -> int:
        """
        Calculates the *estimated* amount of blocks that will be present in `self.rgb_targets` this is used for warning
        about an inadequate amount of blocks.
        :return: Estimated number of blocks.
        """
        width, height = self.target.size
        width_t = math.ceil(width / self.inverse_resolution) * self.inverse_resolution
        height_t = math.ceil(height / self.inverse_resolution) * self.inverse_resolution

        return (width_t//self.inverse_resolution) * (height_t//self.inverse_resolution)

    def overlay_blocks(self,
                       blur: bool = False, blur_strength: int = 10,
                       color_overlay: bool = False, overlay_strength: int = 200
                       ) -> Image.Image:
        logger.debug("Overlaying blocks.")
        """
        Overlays the closest blocks onto a blank image.
        :param blur: Whether to blur each block, can improve quality while removing the zoomed in quality.
        :param blur_strength: How strong to blur the image
        :param overlay_strength:
        :param color_overlay:
        :return:
        """
        new_target = Image.new("RGBA", self.target.size)
        new_target.resize((self.processed_width, self.processed_height))

        idx = 0
        for current_height in range(0, self.processed_height, self.inverse_resolution):
            for current_width in range(0, self.processed_width, self.inverse_resolution):
                if idx < len(self.closest):
                    block = self.closest[idx].im
                    if blur:
                        block = blur_im(block, strength=blur_strength)

                    block = block.resize((self.inverse_resolution, self.inverse_resolution)).convert("RGBA")

                    new_target.paste(block, (current_width, current_height))

                    if color_overlay:
                        color = Image.new("RGB", block.size, self.rgb_targets[idx])
                        mask = Image.new("RGBA", block.size, (0, 0, 0, 255-overlay_strength))
                        final = Image.composite(block, color, mask)

                        new_target.paste(final, (current_width, current_height))
                    idx += 1
        return new_target

    def dump_closest(self, path: Union[pathlib.Path, None]) -> None:
        """
        Dumps all the closest images sequentially to a given path.
        :param path: Where to dump the files, **make sure the directory exists**.
        :return: None
        """
        for k, v in enumerate(self.closest):
            v.save(os.path.join(path, f"{k}.jpg"))


if __name__ == '__main__':
    # setup logger
    import sys

    logging.basicConfig(stream=sys.stdout, level=9)

    import os
    import time
    from concurrent.futures import ThreadPoolExecutor

    RELOAD_IMAGES = False
    INVERSE_RESOLUTION = None  # None to calculate
    MAX_WORKERS = 25
    SRC = "../prompt_data"
    GOAL = "./goal.png"
    MAX_IMAGES = 10000  # None for all images
    PERCENTAGE_TO_INCLUDE = 1

    BLUR_STRENGTH = 1
    OVERLAY_STRENGTH = 100

    image_count = len(os.listdir(SRC))

    goal = Image.open(GOAL)
    inverse_target = calculate_optimal_inverse_resolution(goal,
                                                          int(min(MAX_IMAGES, image_count) * PERCENTAGE_TO_INCLUDE)
                                                          )
    logger.debug(f"Calculated an inverse resolution of {inverse_target}x{inverse_target}")

    logger.info("Loading images.")

    start_load_time = time.time()
    if RELOAD_IMAGES:
        # Load images using multiple workers.
        with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
            imgs = list(executor.map(
                lambda path: MosaicImageChild(os.path.join(SRC, path), inverse_target),
                os.listdir(SRC)[:MAX_IMAGES]
            ))
        end_load_time = time.time()
        logger.info(f"(re)Loaded images in {end_load_time-start_load_time:.2f} seconds.")
        with open("../images.pickle", 'wb') as f:
            pickle.dump(imgs, f)
    else:
        with open("../images.pickle", 'rb') as f:
            imgs = pickle.load(f)
        end_load_time = time.time()
        logger.info(f"Loaded images from pickle dump in {end_load_time-start_load_time:.2f} seconds.")

    logger.info("Loaded images")

    logger.debug("Creating MosaicImage object.")
    mosaic = MosaicImage(goal, imgs, INVERSE_RESOLUTION)

    logger.info("Creating color grids")
    start_color_grid_time = time.time()
    mosaic.create_color_grids()
    end_color_grid_time = time.time()

    logging.info(f"Created color grids in {end_color_grid_time-start_color_grid_time:.2f} seconds.")

    logger.info("Compiling images.")
    start_compile_time = time.time()
    mosaic.compile_closest_images()
    end_compile_time = time.time()

    logger.info(f"Compiled images in {end_compile_time-start_compile_time:.2f} seconds.")

    logger.info("Generating final image.")
    final_basic = mosaic.overlay_blocks(blur=False)

    logger.info("Writing final image.")
    final_basic.save("./final.png")
    logger.info("Saved image to \"./final.png\".")

    logger.info("Generating final (blurred) image.")
    final_blurred = mosaic.overlay_blocks(blur=True, blur_strength=BLUR_STRENGTH)
    final_blurred.save("./final_blurred.png")
    logger.info("Saved image to \"./final_blurred.png\".")

    logger.info("Generating final (color overlay) image.")
    final_color_overlay = mosaic.overlay_blocks(color_overlay=True, overlay_strength=OVERLAY_STRENGTH)
    final_color_overlay.save("./final_color_overlay.png")
    logger.info("Saved image to \"./final_color_overlay.png\".")

    logger.info("Generating final (blurred and color overlay) image.")
    final_blurred_color_overlay = mosaic.overlay_blocks(
        blur=True, blur_strength=BLUR_STRENGTH,
        color_overlay=True, overlay_strength=OVERLAY_STRENGTH
    )
    final_blurred_color_overlay.save("./final_blurred_color_overlay.png")
    logger.info("Saved image to \"./final_blurred_color_overlay.png\".")

    blurred_goal = blur_im(goal, 5)
    blurred_goal = blurred_goal.resize(final_blurred_color_overlay.size)
    blurred_goal.save("original_blurred.png")