#!/usr/bin/env python3
"""Strip the dark void-color background by chroma-distance, not pure
luminance. The portraits are RGB and the "background" is the noesis
void color (#070B1D-ish, RGB ~ 1, 4, 25). Pichet's deep violet armor
has dark pixels too, so a pure luminance threshold either misses the
bg fringe or eats the armor. Chroma-distance from the sampled corner
color lets us drop the bg cleanly without harming the figure.

For each PNG we:
  1. Sample the four corners → bg_color (median)
  2. For each pixel, compute Euclidean distance from bg_color in RGB
  3. Smooth alpha ramp: distance ≤ NEAR → fully transparent,
     distance ≥ FAR → fully opaque, linear in between

NEAR/FAR are tuned per-character because Pichet's armor sits closer
in color-space to the bg than Aletheios's robe does.
"""

from PIL import Image
from pathlib import Path
from math import sqrt

# Per-character distance tuning. Tighter NEAR/FAR for Pichet because
# her violet armor sits closer to the void color than Aletheios's
# brighter robe palette.
THRESHOLDS = {
    "pichet":    {"near": 18, "far": 38},
    "aletheios": {"near": 22, "far": 50},
}

def sample_bg(im):
    w, h = im.size
    px = im.load()
    corners = [
        px[0, 0], px[w - 1, 0], px[0, h - 1], px[w - 1, h - 1],
        # also a few inset corner pixels
        px[5, 5], px[w - 6, 5], px[5, h - 6], px[w - 6, h - 6],
    ]
    # median per channel
    r = sorted(c[0] for c in corners)[len(corners) // 2]
    g = sorted(c[1] for c in corners)[len(corners) // 2]
    b = sorted(c[2] for c in corners)[len(corners) // 2]
    return (r, g, b)

def alpha_for(rgb, bg, near, far):
    dr = rgb[0] - bg[0]
    dg = rgb[1] - bg[1]
    db = rgb[2] - bg[2]
    d = sqrt(dr * dr + dg * dg + db * db)
    if d <= near:
        return 0
    if d >= far:
        return 255
    return int(((d - near) / (far - near)) * 255)

def process(path: Path) -> tuple[int, int, tuple[int, int, int]]:
    im = Image.open(path)
    if im.mode != "RGB":
        im = im.convert("RGB")
    bg = sample_bg(im)
    character = "pichet" if "pichet" in path.name else "aletheios"
    near = THRESHOLDS[character]["near"]
    far = THRESHOLDS[character]["far"]
    rgba = Image.new("RGBA", im.size)
    src = im.load()
    dst = rgba.load()
    transparent = 0
    for y in range(im.size[1]):
        for x in range(im.size[0]):
            r, g, b = src[x, y]
            a = alpha_for((r, g, b), bg, near, far)
            dst[x, y] = (r, g, b, a)
            if a == 0:
                transparent += 1
    rgba.save(path, "PNG", optimize=True)
    return transparent, im.size[0] * im.size[1], bg

ROOTS = [
    Path("/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/sankalpa/public/depth-reading/characters"),
]

for root in ROOTS:
    print(f"\n=== {root} ===")
    for png in sorted(root.glob("*.png")):
        t, total, bg = process(png)
        pct = 100.0 * t / total
        print(f"  {png.name:30s}  bg={bg}  transparent={pct:5.1f}%")
