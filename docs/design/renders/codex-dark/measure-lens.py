import sys
import numpy as np
from PIL import Image

def box_blur(a, k):
    # separable box blur via cumulative sums, k odd
    pad = k // 2
    p = np.pad(a, pad, mode="constant")
    c = np.cumsum(np.cumsum(p, axis=0), axis=1)
    c = np.pad(c, ((1, 0), (1, 0)), mode="constant")
    h, w = a.shape
    s = c[k:k + h, k:k + w] - c[0:h, k:k + w] - c[k:k + h, 0:w] + c[0:h, 0:w]
    return s / (k * k)

def lens(path, k=None):
    im = np.asarray(Image.open(path).convert("RGB")).astype(float) / 255.0
    h, w, _ = im.shape
    r, g, b = im[..., 0], im[..., 1], im[..., 2]
    mx = im.max(axis=2); mn = im.min(axis=2)
    sat = np.where(mx > 0, (mx - mn) / np.maximum(mx, 1e-6), 0)
    mask = ((g > r * 1.6) & (b > r * 1.4) & (g > 0.25) & (sat > 0.45) & (mx > 0.3)).astype(float)
    if mask.sum() < 20:
        return None
    k = k or max(9, int(min(h, w) * 0.04) | 1)
    dens = box_blur(mask, k)
    y0, x0 = np.unravel_index(np.argmax(dens), dens.shape)
    rad = k
    ys, xs = np.nonzero(mask[max(0, y0 - rad):y0 + rad + 1, max(0, x0 - rad):x0 + rad + 1])
    ys = ys + max(0, y0 - rad); xs = xs + max(0, x0 - rad)
    return round(xs.mean() / w * 100, 1), round(ys.mean() / h * 100, 1), int(mask.sum()), (w, h)

if __name__ == "__main__":
    for p in sys.argv[1:]:
        print(p.split("/")[-1], lens(p))
