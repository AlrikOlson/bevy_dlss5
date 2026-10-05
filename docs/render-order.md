# External display composition

`DlssNeuralRenderingSet` identifies the feature-18 evaluation system in the `Core3d` schedule. It remains in `Core3dSystems::PostProcess` after Bevy's `tonemapping`. Applications can order a physical display reference before this set and restore scientifically constrained pixels afterward, without disabling neural evaluation or depending on a private function.

Space-sim integrates this boundary with Solarik's `CameraOpticsSystems::ExternalDisplayEffects`. Native Vulkan/DLSS5 captures verified successful feature-18 evaluation and before/after radiometric composition. The boundary alone changes no default rendering or component values.
