# README capture record

September 6, 2026: all four README images are refreshed with Solarik `cc1f9e6`
(GI foliage endpoints), and playground `90f532a`. These are
development renderer captures beyond the tagged Solarik release. The matching
Bistro exterior and original cafe interior are loaded.

Both modes use Solarik, native 1920x1080 DLSS Ray Reconstruction, preset
exposure and lights, fog/blur, and no additional grading. Day is frame 750 of
the 1800-frame timeline with 512 fixed-pose warmup frames. Dusk is frame 1520
with 1024 warmup frames. The historical `night` filenames contain dusk views.
The full HUD images have Neural Rendering ON; runtime logs confirm it was
active. Capture logs contain no shader or validation errors.

The comparison images preserve the source pixels on each side of the diagonal,
adding only a divider and a 100-pixel external label strip. The lower-right HUD
comes from the ON image. All four final assets were inspected at full resolution.
Independent runs retain stochastic lighting differences. HUD frame times include
capture overhead and are not an ON/OFF performance benchmark.

Fixed-camera and fixed-lighting settling checks use the NR-off source images.
The metric is mean sRGB-decoded display-linear luminance with weights
(0.2126, 0.7152, 0.0722):

| View / region | Rectangle x, y, width, height | Warmup comparison | Change |
| --- | --- | --- | ---: |
| Day pavement | 215, 965, 275, 95 | 128 to 512 | +0.22% |
| Day facade | 1125, 50, 450, 225 | 128 to 512 | +0.54% |
| Day wall | 555, 590, 250, 165 | 128 to 512 | +2.83% |
| Dusk pavement | 50, 860, 380, 125 | 512 to 1024 | +2.30% |
| Dusk facade | 65, 75, 335, 225 | 512 to 1024 | +0.90% |
| Dusk wall | 1075, 830, 255, 140 | 512 to 1024 | +0.96% |

These bounded checks do not establish full convergence. Shadow variation and
soft foliage remain. Solarik's separate README night control still shows
variation after a longer warmup.

The separate local capture rig was release-built with `cargo build --release --locked`.
From that rig in Git Bash, the ON sources can be reproduced with:

```bash
SCENE=bistro SCENE_HUD=1 NR_RES=1920x1080 NR_START=750 NR_TIMELINE=1800 NR_CAPTURE=1 NR_WARMUP=512 SCENE_FX=fog,blur NR_OUT=../artifacts/bevy-sponza/fe-hud-day-on-512 python tools/launch.py capture on
SCENE=bistro SCENE_HUD=1 NR_RES=1920x1080 NR_START=1520 NR_TIMELINE=1800 NR_CAPTURE=1 NR_WARMUP=1024 SCENE_FX=fog,blur NR_OUT=../artifacts/bevy-sponza/fe-hud-night-on-1024 python tools/launch.py capture on
```

Repeat with `capture off` and a distinct output directory for the OFF sources.
Local `fe-*` capture/composition scripts, logs and JSON receipts stay under the
organizer's `artifacts/bevy-sponza/`. The inventory verifier checks all 12 README
image files across the three active repositories, including byte-identical
copies of these four images in the playground. The README retains scene credits.
