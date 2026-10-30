# Lessons

- **Check Gene Keys sphere sources against the user's own chart.** IQ = P Venus, EQ = P Mars, SQ = D Venus, Vocation = D Mars. An early draft swapped these.
- **Use the true node for Human Design.** The mean node changed the Design Node gate and the defined centers; verify against the user's bodygraph image.
- **Verify transit claims by computation, never from memory or a prior reading.** The previous reading's Jupiter, Mars and Variables claims were all wrong for the date.
- **Run `extra.py` after `daily.py`.** `daily.py` rewrites `sky.json` from scratch.
- **Dates shown to the user are IST.** Compute events in UT, convert once.
- **The engine is the source of truth for Vedic periods.** Selemene's Mahadasha start (14 Sep 2026) beats a local calculation (13 Sep).
- **The edition history is context, not content.** Never surface "what changed" or "previous reading" language in the artifact.
- **Never name a scene method after a kit method.** The orrery's `setView` overwrote the stage's and recursed forever on the Orbit/Above toggle.
- **A stage callback can fire during construction.** Declare the handle with `let S=null` before `S=K.stage(...)`; `onResize` runs inside the constructor.
- **Fit the camera to the projected object, not the bounding radius.** A tilted ring is flatter than it is wide; fitting its radius to the vertical field of view leaves the instrument small.
