# README capture record

## Current GPU atmosphere images

September 6, 2026 (local): captured from the integrated release build containing
the GPU clear-sky atmosphere, stable fixed-camera interpolation and shared
HUD/diagnostic registration, normal capture shutdown and explicit NR resource
cleanup before runtime unload. Source bases are Solarik `cc1f9e6`, playground
`460be24` and DLSS5 `4f37244`, plus this iteration's changes.
Executable SHA-256:
`b5004c45dea2b72df878318d95b88f13b9144838c55f3d53693725259ef21235`.
Windows/Vulkan on RTX 4090, NVIDIA Studio driver 616.56; GPU timestamp
instrumentation disabled for these images. Build with
`cargo build --release --locked` in the playground. After capture, the staged
whitespace check removed exactly one trailing newline from `model.wgsl`;
all other source bytes match the build manifest. The preserved captured shader
and final artifact audit verify that sole formatting difference.

The original Bistro exterior and matching cafe interior are loaded. Both
render modes use Solarik and native-resolution DLSS Ray Reconstruction.
The atmosphere uses the High preset quality, density scales 1, Mie g 0.8,
2,000 m aerial distance and IBL scale 1. Camera exposure follows the authored
12.5/5.0 EV100 day/night curve. Local artistic fog and extra grading are off;
`SCENE_FX=blur` retains the usual camera effects. Camera and lighting stay
fixed during warmup. Historical `night` filenames below contain dusk views.

All four README images use 1920×1080 rendering: day at timeline 750 / 1800
with 512 warmup frames, dusk at 1520 / 1800 with 1,024 warmup frames.
Both full HUD images have Neural Rendering ON. The comparison sources use
matching camera, exposure and warmup with NR OFF/ON in separate runs.

Independent warmups hold camera and lighting fixed. The metric is regional
mean sRGB-decoded display-linear luminance with weights (0.2126, 0.7152, 0.0722).
Absolute changes use the same 0–1 display-linear scale.

| View / region | Rectangle x, y, width, height | Warmup comparison | Luminance change | Absolute change |
| --- | --- | --- | ---: | ---: |
| HUD day pavement | 215, 965, 275, 95 | 128 to 512 | -1.12% | -0.000207 |
| HUD day facade | 1125, 50, 450, 225 | 128 to 512 | -1.81% | -0.000558 |
| HUD day wall | 555, 590, 250, 165 | 128 to 512 | +0.34% | +0.000029 |
| HUD dusk pavement | 50, 860, 380, 125 | 512 to 1024 | +0.88% | +0.000087 |
| HUD dusk facade | 65, 75, 335, 225 | 512 to 1024 | +1.44% | +0.001054 |
| HUD dusk wall | 1075, 830, 255, 140 | 512 to 1024 | +0.10% | +0.000003 |

These NR-off regional means change at most 1.81% by day and 1.44% at dusk.
This checks the underlying lighting warmup; it does not establish full convergence
or quantify NR reconstruction accuracy. All final HUD and comparison images
were inspected. Shadow variation and foliage softness remain.

The complete set of 16 serial captures exited normally, produced fresh PNGs
and had no shader/validation errors or Windows NVIDIA driver events. Both NR
ON logs confirm active evaluation and explicit feature release before shutdown.

The comparison images preserve source pixels on each side of the diagonal.
Only a three-pixel divider and a 100-pixel external label strip are added.
The lower-right HUD comes from the ON image. Stochastic lighting differences
remain between runs, and HUD frame times include capture overhead; these
pictures do not benchmark the isolated cost of Neural Rendering.

After the release build, reproduce the ON sources from the playground in Git Bash:

```bash
SCENE=bistro SCENE_HUD=1 NR_RES=1920x1080 NR_START=750 NR_TIMELINE=1800 NR_CAPTURE=1 NR_WARMUP=512 SCENE_FX=blur SCENE_DIAG=0 NR_OUT=../artifacts/bevy-sponza/sky-hud-day-on-512 python tools/launch.py capture on
SCENE=bistro SCENE_HUD=1 NR_RES=1920x1080 NR_START=1520 NR_TIMELINE=1800 NR_CAPTURE=1 NR_WARMUP=1024 SCENE_FX=blur SCENE_DIAG=0 NR_OUT=../artifacts/bevy-sponza/sky-hud-night-on-1024 python tools/launch.py capture on
```

Repeat with `capture off` and distinct output directories for the OFF sources.
Local `sky_*` scripts and `sky-*.json` receipts live under the organizer's
`artifacts/bevy-sponza/`. The inventory verifies all 12 README images in the
three active repositories and byte-identical copies of these four assets in
the playground. Scene attribution remains in the README.

## Earlier GI foliage endpoint images

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
