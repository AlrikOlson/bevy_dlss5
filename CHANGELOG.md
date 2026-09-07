# Changelog

## Unreleased

- Keep the render device alive through NR shutdown, wait for GPU work, and
  release feature handles and bridge textures before unloading the runtime DLL.
- Handle application exit in render cleanup while other NGX consumers remain alive.

## 0.1.1 — 2026-09-06

First tagged release.

- DLSS 5 Neural Rendering for Bevy 0.19.1, with camera controls and temporal reset.
- Runs after tonemapping, with color conversion around the NGX call.
- Includes a small scene for trying NR and toggling it with **N**.
- Cleans up the standalone build and fixes the Clippy warning in the color bridge.

Tested on Windows, Vulkan and an RTX 4090. You need an NGX SDK checkout to link the crate and an NR runtime DLL built for your GPU to run it. See the [README](README.md#requirements) for setup and known issues.
