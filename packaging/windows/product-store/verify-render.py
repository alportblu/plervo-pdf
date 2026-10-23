"""Independently decode a native installed-app export; reject blank output."""
import sys
from PIL import Image, ImageStat

with Image.open(sys.argv[1]) as image:
    image.load()
    if image.format != "PNG" or min(image.size) < 32:
        raise ValueError("Expected a usable PNG render")
    if max(ImageStat.Stat(image.convert("RGB")).stddev) < 1:
        raise ValueError("Export is blank")
    print(f"Decoded {image.width}x{image.height} nonblank PNG")
