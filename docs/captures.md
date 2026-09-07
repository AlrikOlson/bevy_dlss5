# README capture record

## Current images from the weapon inspection build

September 7, 2026 (local): refreshed from the integrated release build containing
the weapon import and inspection scene. Source bases: playground `fc946ff`,
Solarik `c531ee0` and DLSS5 `ee359d5`, plus the weapon inspection changes.
The Bistro scene and renderer source are unchanged by this iteration.
Executable SHA-256:
`44d81039a0bb8c47cd5cc61dddf11f794ebfdaa46e1c4a0a9bfd75b2e62bb0c1`.
Windows/Vulkan, RTX 4090, NVIDIA Studio driver 616.56; build with
`cargo build --release --locked` in the playground.

These images retain the corrected exterior/interior import described below.
GPU atmosphere uses High quality, density scales 1, Mie g 0.8, 2,000 m aerial
distance and IBL scale 1. Native-resolution DLSS Ray Reconstruction is active.
The preset supplies exposure (12.5/5.0 EV100 day/night) and lamps.
`SCENE_FX=blur` and `SCENE_DIAG=0`; every warmup holds camera and lighting fixed.
Historical `night` filenames contain dusk views.

All four images render at 1920×1080. The day pose is timeline 750 / 1800
with 512 warmup frames; dusk is 1520 / 1800 with 1,024. Both full HUD images
have NR on. Comparisons use matching pose, exposure and warmup with NR off/on.
Matching playground copies are byte-identical.

Independent warmups measure mean sRGB-decoded display-linear luminance with
weights (0.2126, 0.7152, 0.0722). Absolute changes use the same 0–1 scale.

| View / region | Rectangle x, y, width, height | Warmup frames | Change | Absolute change |
| --- | --- | --- | ---: | ---: |
| HUD day pavement | 215, 965, 275, 95 | 128 to 512 | +1.39% | +0.000258 |
| HUD day facade | 1125, 50, 450, 225 | 128 to 512 | -0.19% | -0.000060 |
| HUD day wall | 555, 590, 250, 165 | 128 to 512 | +2.38% | +0.000132 |
| HUD dusk pavement | 50, 860, 380, 125 | 512 to 1024 | +0.60% | +0.000061 |
| HUD dusk facade | 65, 75, 335, 225 | 512 to 1024 | -0.01% | -0.000009 |
| HUD dusk wall | 1075, 830, 255, 140 | 512 to 1024 | -1.04% | -0.000039 |

The diagonal comparisons use the OFF source on the left and ON on the right,
preserving source pixels except for the narrow divider and a 100-pixel external
label strip. The lower-right HUD comes from the ON source. Separate runs retain
stochastic lighting differences; HUD timings include capture overhead and do
not benchmark isolated Neural Rendering cost.


All final images were inspected. All 14 serial Bistro runs produced fresh PNGs,
exited normally and had no shader or validation errors. NR ON logs confirm active
evaluation and explicit feature release. The organizer's local
`artifacts/bevy-sponza/weapon_import_capture.py` records exact launch environments
and image hashes; `weapon_import_audit.py` records source/binary identity,
regional settling and all 12 README image paths with matching duplicate hashes.
Its receipts are `wii-build.json`, `wii-settling.json`,
`wii-readme-inventory.json` and `wii-final-audit.json`. The latter records one
nonsemantic difference after capture: final whitespace review removed a trailing
blank line from the weapon preset, verified by restoring that single newline
and matching its captured SHA-256. All executable sources and scene values match.

These bounded samples do not establish full convergence. Dark interiors,
foliage softness and stochastic spatial variation remain. The night control's
cafe wall changes -3.17% (-0.000166) from 2,048 to 4,096 warmup frames; the
existing night-image-quality follow-up remains open.

## Historical images with the corrected Bistro interior

September 6, 2026 (local): refreshed from the final integrated release build
after repairing the Bistro interior import. Source bases: Solarik `df9a05a`,
DLSS5 `e8b2703` and playground `740e78a`, plus the paired-import fix.
Executable SHA-256:
`3a996f4bad28d5fc5ed428e41546ec78a9c65924406b2450d9844cf25b514056`.
Windows/Vulkan, RTX 4090, NVIDIA Studio driver 616.56. Build with
`cargo build --release --locked` in the playground.

The qian-o exterior needs import scale **0.625**; its interior needs **1.0**.
The old recipe incorrectly applied 0.625 to both. Reimporting aligns eight
shared facade anchors within 0.004 mm and makes the audited window rays hit
real room surfaces. Those earlier bright window patches were empty-world
visibility, not emissive interiors. Run
`python tools/import_bistro.py "C:/scenes/GLTF-Assets/Bistro"` in the playground
to repair an old local import. Its `docs/bistro-glass.md` contains the
before/after crops and ray-distance regression.

Solarik's GPU atmosphere and native-resolution DLSS Ray Reconstruction remain
active. The atmosphere uses High quality, density scales 1, Mie g 0.8,
2,000 m aerial distance and IBL scale 1. Exposure uses the preset's
12.5/5.0 EV100 day/night curve; scene lamps are unchanged. Extra grading and
local artistic fog are disabled (`SCENE_FX=blur`); GPU timestamp diagnostics
are off. Camera and lighting stay fixed throughout warmup. Historical
`night` filenames contain dusk views.

All four README images use 1920×1080 rendering: timeline 750 / 1800 with
512 fixed-pose warmup frames by day, 1520 / 1800 with 1,024 at dusk.
Both full HUD images have Neural Rendering on. The comparison sources use
matching camera, lighting, exposure, resolution and warmup with NR off/on.

Independent warmups use identical camera and lighting. The metric is regional
mean sRGB-decoded display-linear luminance, with weights (0.2126, 0.7152, 0.0722).
Absolute changes are on the same 0–1 display-linear scale.

| View / region | Rectangle x, y, width, height | Warmup comparison | Luminance change | Absolute change |
| --- | --- | --- | ---: | ---: |
| HUD day pavement | 215, 965, 275, 95 | 128 to 512 | +1.03% | +0.000186 |
| HUD day facade | 1125, 50, 450, 225 | 128 to 512 | +0.45% | +0.000138 |
| HUD day wall | 555, 590, 250, 165 | 128 to 512 | +3.32% | +0.000272 |
| HUD dusk pavement | 50, 860, 380, 125 | 512 to 1024 | +0.37% | +0.000037 |
| HUD dusk facade | 65, 75, 335, 225 | 512 to 1024 | +1.25% | +0.000914 |
| HUD dusk wall | 1075, 830, 255, 140 | 512 to 1024 | +1.13% | +0.000035 |

The HUD regions change at most 3.32% by day and 1.25% at dusk.
The separate Solarik night control still has a −5.82% cafe-wall change from
2,048 to 4,096 frames, recorded with the existing night-quality follow-up.

The diagonal comparisons preserve source pixels in each half. Only a
three-pixel divider and a 100-pixel external label strip are added. The
lower-right HUD comes from the ON capture. Separate runs retain stochastic
lighting differences, and HUD timings include capture overhead; the images
do not benchmark the isolated cost of Neural Rendering.

From the playground in Git Bash, after the paired import and release build:

```bash
SCENE=bistro SCENE_HUD=1 NR_RES=1920x1080 NR_START=750 NR_TIMELINE=1800 NR_CAPTURE=1 NR_WARMUP=512 SCENE_FX=blur SCENE_DIAG=0 NR_OUT=../artifacts/bevy-sponza/glass-leak-hud-day-on-512 python tools/launch.py capture on
SCENE=bistro SCENE_HUD=1 NR_RES=1920x1080 NR_START=1520 NR_TIMELINE=1800 NR_CAPTURE=1 NR_WARMUP=1024 SCENE_FX=blur SCENE_DIAG=0 NR_OUT=../artifacts/bevy-sponza/glass-leak-hud-night-on-1024 python tools/launch.py capture on
```

Repeat with `capture off` and distinct output directories for the OFF sources.

All final images were inspected. These bounded regional comparisons do not
establish full convergence; dark interiors, foliage softness and stochastic
spatial variation remain.

All 18 serial captures produced fresh PNGs, exited normally and had
no shader/validation errors or Windows NVIDIA driver events. Both NR ON logs
confirm active evaluation and explicit feature release before runtime shutdown.
All runtime source and imported geometry hashes match the capture manifest.
The only removed manifest entry was an unused standalone interior preset
created by the importer; the assembled Bistro loads the interior glTF directly.

The organizer's local `artifacts/bevy-sponza/glass_leak_readme_capture.py`
retains the recipes and `glass-leak-*/capture-result.json` the exact settings.
`glass-leak-build.json`, `glass-leak-settling.json`,
`glass-leak-driver-audit.json` and `glass-leak-final-audit.json` record the
source/asset hashes, measurements and checks. `glass-leak-readme-inventory.json`
verifies all **12** README image files across the three active repositories,
including details blocks and matching duplicates. The four comparison/HUD
images are byte-identical in the playground and DLSS5 repositories.

## Earlier GPU atmosphere images (undersized interior)

The following record is historical. Its incorrect interior scale allowed
empty-world visibility behind some cafe panes. The images referenced by the
README have all been replaced by the corrected assembly above.

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
