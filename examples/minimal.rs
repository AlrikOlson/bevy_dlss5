//! Minimal bevy_dlss5 integration: a lit scene with DLSS 5 Neural Rendering on
//! the camera.
//!
//! Build, then run the binary renamed to `nvngx.dll` (the NGX caller check), with
//! the Vulkan backend selected. Provide the runtime either as a single
//! `nvngx_dlssnr.dll` next to the exe, or as a `dlssnr/` arch folder next to the
//! exe (the plugin auto-selects the build matching your GPU):
//!
//! ```powershell
//! cargo build --release --example minimal
//! copy target\release\examples\minimal.exe target\release\examples\nvngx.dll
//! # then place nvngx_dlssnr.dll (or a dlssnr\ folder) next to nvngx.dll
//! $env:WGPU_BACKEND="vulkan"; target\release\examples\nvngx.dll
//! ```
//!
//! Press N to toggle neural rendering on the camera.

use bevy::prelude::*;
use bevy_dlss5::{DlssNeuralRendering, DlssNeuralRenderingPlugin, DlssNeuralRenderingStatus};

fn main() {
    App::new()
        // Before DefaultPlugins: it registers the Vulkan device extensions.
        .add_plugins(DlssNeuralRenderingPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle, report))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A few PBR shapes to show the neural lighting detail.
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.5),
        metallic: 1.0,
        perceptual_roughness: 0.25,
        ..default()
    });
    let floor = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.32),
        perceptual_roughness: 0.9,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(floor),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.6).mesh().uv(48, 24))),
        MeshMaterial3d(metal.clone()),
        Transform::from_xyz(0.0, 0.6, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 0.7))),
        MeshMaterial3d(metal),
        Transform::from_xyz(1.3, 0.35, -0.4),
    ));

    commands.spawn((
        DirectionalLight { illuminance: 6_500.0, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(3.0, 5.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        // Hdr, DepthPrepass and MotionVectorPrepass are added automatically.
        DlssNeuralRendering::default(),
        // Storage-based in-place write needs TAA, not MSAA, for anti-aliasing.
        bevy::anti_alias::taa::TemporalAntiAliasing::default(),
        Msaa::Off,
        Transform::from_xyz(2.5, 1.6, 3.2).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}

/// Toggle the component to turn neural rendering off/on for the camera.
fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    on: Query<Entity, (With<Camera3d>, With<DlssNeuralRendering>)>,
    off: Query<Entity, (With<Camera3d>, Without<DlssNeuralRendering>)>,
) {
    if !keys.just_pressed(KeyCode::KeyN) {
        return;
    }
    for e in &on {
        commands.entity(e).remove::<DlssNeuralRendering>();
        info!("neural rendering OFF");
    }
    for e in &off {
        commands.entity(e).insert(DlssNeuralRendering::default());
        info!("neural rendering ON");
    }
}

fn report(status: Res<DlssNeuralRenderingStatus>, mut done: Local<bool>) {
    if *done {
        return;
    }
    if status.is_active() {
        info!("bevy_dlss5 example: neural rendering is ACTIVE");
        *done = true;
    } else if status.is_unavailable() {
        warn!("bevy_dlss5 example: neural rendering unavailable (see earlier warning)");
        *done = true;
    }
}
