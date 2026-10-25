# Image fixtures

| File | Size | Origin | Used by |
|---|---|---|---|
| `biofield-portrait-256.jpg` | 256 px longest edge, JPEG q85, < 60 KB | Downscaled from a **synthetic, AI-generated portrait** (`synthetic-face.png`, produced for the lane S1 face-reading fixture). It depicts no real person. | `crates/noesis-api/tests/biofield_capture_proxy.rs` and the staging capture probes in `docs/engines/biofield.md`. |

Rules:

- Only synthetic images. Never add a photo of a real person to this folder.
- Keep each file under 60 KB so the test crates stay small.
