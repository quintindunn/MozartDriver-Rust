from PIL.Image import Image
from PIL.Image import open as im_open
from PIL import ImageFilter


import logging


logger = logging.getLogger(__name__)


class MosaicImageChild:
    """
    NOTE: This is a very temperamental class to say the least, a lot of PIL.Image.Image methods will cause attributes to
    be cleared/overwritten.

    Future: Refactor the program to instead use something like MosaicImageChild.im instead of utilizing inheritance.
    """
    def __init__(self, fp, resolution: int):
        if isinstance(fp, Image):
            fp.convert("RGB")
            self.im = fp
        else:
            im = im_open(fp).convert("RGB")
            self.im = im

            if self.im.size[0] != resolution and self.im.size[1] != resolution:
                self.im = self.im.resize((resolution, resolution))

        logger.debug("Calculating average RGB value for image.")
        self.avg_rgb = self._calculate_average_rgb()
        self.resolution = resolution

    def _calculate_average_rgb(self) -> tuple[int, int, int]:
        """
        Calculates the average RGB value of the image.
        :rtype: tuple[int, int, int]
        :return:
        """
        ar, ag, ab = 0, 0, 0
        k = 0
        for k, (r, g, b) in enumerate(self.im.getdata()):
            ar += r
            ag += g
            ab += b

        return ar // k, ag // k, ab // k


def blur_im(im, strength: int = 10):
    return im.filter(ImageFilter.GaussianBlur(strength))
