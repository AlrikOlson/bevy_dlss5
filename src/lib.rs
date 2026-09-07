#![doc = include_str!("../README.md")]

mod bridge;
mod ngx;
pub mod patch;

use bevy::app::AppExit;
use bevy::camera::{CameraMainTextureUsages, Hdr};
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, ViewPrepassTextures};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::core_pipeline::{Core3d, Core3dSystems};
use bevy::log::{info, warn};
use bevy::prelude::*;
use bevy::render::render_resource::{CommandEncoderDescriptor, TextureUsages};
use bevy::render::renderer::raw_vulkan_init::RawVulkanInitSettings;
use bevy::render::renderer::{RenderAdapter, RenderContext, RenderDevice, ViewQuery};
use bevy::render::sync_component::SyncComponent;
use bevy::render::sync_component::SyncComponentPlugin;
use bevy::render::sync_world::{MainEntity, RenderEntity};
use bevy::render::view::ViewTarget;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems};

use bridge::Bridge;
use ngx::{NgxCore, ResourceVk};
use std::collections::HashMap;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

/// A default NGX application/project id (any valid UUID works). Override it in
/// [`DlssNeuralRenderingPlugin`] with an id NVIDIA issued you, if you have one.
pub const DEFAULT_PROJECT_ID: &str = "a0f57b54-1daf-4934-90ae-c4035c19df04";

const VK_FORMAT_R16G16B16A16_SFLOAT: u32 = 97;
const DLSSNR_DLL_NAME: &str = "nvngx_dlssnr.dll";

// ---------------------------------------------------------------------------
// Architecture selection
// ---------------------------------------------------------------------------

/// The GPU-architecture tier a `nvngx_dlssnr.dll` is built for.
///
/// The neural-rendering kernels are architecture-specific compiled device code,
/// so the DLL must match the GPU. These names mirror the community per-arch
/// build layout (`Turing+`, `Ada Lovelace+`, `Blackwell+`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchTier {
    /// Turing and Ampere (RTX 20xx / 30xx, GTX 16xx, A-series).
    TuringPlus,
    /// Ada Lovelace (RTX 40xx, L40/L4, RTX Ada).
    AdaLovelacePlus,
    /// Blackwell (RTX 50xx, B-series). Genuine, needs no gate patch.
    BlackwellPlus,
}

impl ArchTier {
    /// The per-arch subfolder name in the community DLL layout.
    pub fn folder(self) -> &'static str {
        match self {
            ArchTier::TuringPlus => "Turing+",
            ArchTier::AdaLovelacePlus => "Ada Lovelace+",
            ArchTier::BlackwellPlus => "Blackwell+",
        }
    }

    /// Best-effort detection from a wgpu adapter name (e.g.
    /// `"NVIDIA GeForce RTX 4090"`). Returns `None` for non-NVIDIA or unknown
    /// names.
    pub fn detect(adapter_name: &str) -> Option<Self> {
        let n = adapter_name.to_ascii_uppercase();
        if n.contains("BLACKWELL") {
            return Some(ArchTier::BlackwellPlus);
        }
        if n.contains("ADA") {
            return Some(ArchTier::AdaLovelacePlus);
        }
        // Parse the first GeForce/RTX/GTX model number.
        if let Some(model) = first_model_number(&n) {
            return Some(match model {
                5000..=5999 => ArchTier::BlackwellPlus,
                4000..=4999 => ArchTier::AdaLovelacePlus,
                1600..=3999 => ArchTier::TuringPlus,
                _ => return None,
            });
        }
        None
    }
}

/// Extract the first 3-4 digit model number that follows "RTX" or "GTX".
fn first_model_number(upper: &str) -> Option<u32> {
    for kw in ["RTX", "GTX"] {
        if let Some(i) = upper.find(kw) {
            let rest = &upper[i + kw.len()..];
            let digits: String = rest
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(v) = digits.parse::<u32>() {
                if (1000..=9999).contains(&v) {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Where the plugin finds the `nvngx_dlssnr.dll` runtime.
#[derive(Clone, Debug)]
pub enum DllSource {
    /// Use this exact DLL file, whatever architecture it is built for.
    File(PathBuf),
    /// Auto-pick the arch subfolder inside this directory, using the community
    /// layout: `<dir>/Ada Lovelace+/nvngx_dlssnr.dll`, etc.
    ArchDir(PathBuf),
    /// Look next to the executable: use `nvngx_dlssnr.dll` if it is there, else
    /// treat a `dlssnr/` subfolder as an [`DllSource::ArchDir`].
    Auto,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Enables DLSS 5 Neural Rendering (NGX feature 18) for the app.
///
/// **Add this plugin before [`DefaultPlugins`]** — it registers the Vulkan
/// device extensions feature 18 needs, which must happen before Bevy builds the
/// render device:
///
/// ```ignore
/// App::new()
///     .add_plugins(DlssNeuralRenderingPlugin::default())
///     .add_plugins(DefaultPlugins)
///     // ...
/// ```
///
/// By default the DLL is resolved by [`DllSource::Auto`]: the plugin uses a
/// `nvngx_dlssnr.dll` sitting next to the executable, or auto-selects the
/// architecture-matched build from a `dlssnr/` folder next to the executable.
///
/// If neural rendering is unavailable (no Vulkan RTX GPU, missing/mismatched
/// DLL, or the process is not named `nvngx.dll`) the plugin logs one warning and
/// does nothing else. Your game runs normally. Query
/// [`DlssNeuralRenderingStatus`] to know at runtime whether it went live.
pub struct DlssNeuralRenderingPlugin {
    /// How to locate the `nvngx_dlssnr.dll` runtime.
    pub dll_source: DllSource,
    /// NGX project id (a UUID string). See [`DEFAULT_PROJECT_ID`].
    pub project_id: String,
}

impl Default for DlssNeuralRenderingPlugin {
    fn default() -> Self {
        Self {
            dll_source: DllSource::Auto,
            project_id: DEFAULT_PROJECT_ID.to_string(),
        }
    }
}

impl DlssNeuralRenderingPlugin {
    /// Use one explicit DLL file.
    pub fn with_dll(path: impl Into<PathBuf>) -> Self {
        Self {
            dll_source: DllSource::File(path.into()),
            project_id: DEFAULT_PROJECT_ID.to_string(),
        }
    }
    /// Auto-select the arch-matched DLL from a directory in the community layout.
    pub fn with_arch_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            dll_source: DllSource::ArchDir(dir.into()),
            project_id: DEFAULT_PROJECT_ID.to_string(),
        }
    }
}

/// Resolve a [`DllSource`] to a concrete DLL path, given the GPU adapter name.
fn resolve_dll(source: &DllSource, adapter_name: &str) -> Result<PathBuf, String> {
    let exe_dir = || {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default()
    };
    let from_arch_dir = |dir: &Path| -> Result<PathBuf, String> {
        let tier = ArchTier::detect(adapter_name)
            .ok_or_else(|| format!("could not detect GPU architecture from '{adapter_name}'"))?;
        let path = dir.join(tier.folder()).join(DLSSNR_DLL_NAME);
        if path.exists() {
            Ok(path)
        } else {
            Err(format!(
                "no {} for {tier:?} at {}",
                DLSSNR_DLL_NAME,
                path.display()
            ))
        }
    };
    match source {
        DllSource::File(p) => Ok(p.clone()),
        DllSource::ArchDir(d) => from_arch_dir(d),
        DllSource::Auto => {
            let direct = exe_dir().join(DLSSNR_DLL_NAME);
            if direct.exists() {
                Ok(direct)
            } else {
                from_arch_dir(&exe_dir().join("dlssnr"))
            }
        }
    }
}

/// DLSS 5 Neural Rendering weight preset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DlssNrPreset {
    /// Let the runtime pick (recommended).
    #[default]
    Auto,
    /// Force preset J / K / L / M (content-dependent; experiment).
    J,
    K,
    L,
    M,
}
impl DlssNrPreset {
    fn code(self) -> i32 {
        match self {
            DlssNrPreset::Auto => 0,
            DlssNrPreset::J => 10,
            DlssNrPreset::K => 11,
            DlssNrPreset::L => 12,
            DlssNrPreset::M => 13,
        }
    }
}

/// Add this component to a 3D camera to run Neural Rendering on it.
///
/// It automatically pulls in the camera features feature 18 needs
/// ([`Hdr`], [`DepthPrepass`], [`MotionVectorPrepass`]). Remove the component to
/// turn neural rendering off for that camera. Tweak the fields live at runtime.
#[derive(Component, Clone, Debug)]
#[require(Hdr, DepthPrepass, MotionVectorPrepass)]
pub struct DlssNeuralRendering {
    /// Overall neural effect strength. Typical range `0.0..=2.0`; `1.0` is a
    /// good start.
    pub intensity: f32,
    /// Low-frequency tonal adjustment (broad lighting / color). Keep near `0.0`
    /// to preserve the rendered frame's exposure; the fallback preset largely
    /// ignores it anyway — control brightness via scene exposure instead.
    pub tone: f32,
    /// High-frequency structure (ambient occlusion, contact shadows,
    /// micro-reflections). Typical range `0.0..=2.0`; this is where the visible
    /// enhancement comes from.
    pub structure: f32,
    /// Weight preset.
    pub preset: DlssNrPreset,
    /// Drop the temporal history this frame (a camera cut, a teleport).
    /// Reset automatically to `false` after the frame.
    pub reset: bool,
}

impl Default for DlssNeuralRendering {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            tone: 0.0,
            structure: 1.6,
            preset: DlssNrPreset::Auto,
            reset: false,
        }
    }
}

impl SyncComponent for DlssNeuralRendering {
    type Target = Self;
}

/// Runtime status of Neural Rendering, readable from any system as a resource.
///
/// ```ignore
/// fn ui(status: Res<DlssNeuralRenderingStatus>) {
///     if status.is_active() { /* feature 18 is modifying the frame */ }
/// }
/// ```
#[derive(Resource, Clone)]
pub struct DlssNeuralRenderingStatus(Arc<AtomicU8>);

const STATUS_UNKNOWN: u8 = 0;
const STATUS_ACTIVE: u8 = 1;
const STATUS_UNAVAILABLE: u8 = 2;

impl DlssNeuralRenderingStatus {
    /// True once feature 18 has initialized and is evaluating.
    pub fn is_active(&self) -> bool {
        self.0.load(Ordering::Relaxed) == STATUS_ACTIVE
    }
    /// True if neural rendering was determined unavailable on this machine.
    pub fn is_unavailable(&self) -> bool {
        self.0.load(Ordering::Relaxed) == STATUS_UNAVAILABLE
    }
    fn set(&self, v: u8) {
        self.0.store(v, Ordering::Relaxed);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

impl Plugin for DlssNeuralRenderingPlugin {
    fn build(&self, app: &mut App) {
        // Register the Vulkan device extensions feature 18 needs. This must run
        // before RenderPlugin builds the device — i.e. add this plugin before
        // DefaultPlugins. (The render-world setup happens in `finish()`, which
        // runs once the RenderApp exists.)
        let mut settings = app
            .world_mut()
            .get_resource_or_init::<RawVulkanInitSettings>();
        // SAFETY: we only add extensions/features the adapter reports supported,
        // and we never enable buffer_device_address twice.
        unsafe {
            settings.add_create_device_callback(|args, raw_adapter, _features| {
                let caps = raw_adapter.physical_device_capabilities();
                for ext in [
                    ash::vk::NVX_BINARY_IMPORT_NAME,
                    ash::vk::NVX_IMAGE_VIEW_HANDLE_NAME,
                    ash::vk::KHR_PUSH_DESCRIPTOR_NAME,
                    ash::vk::KHR_MAINTENANCE4_NAME,
                ] {
                    if caps.supports_extension(ext) && !args.extensions.contains(&ext) {
                        args.extensions.push(ext);
                    }
                }
                // buffer_device_address: wgpu often enables VK_KHR_buffer_device_address
                // (and its feature) already. Only add it — as the EXT alias plus a
                // feature struct — when neither is present, to avoid a duplicate
                // sType / conflicting-extension device-creation error.
                let has_bda = args.extensions.iter().any(|e| {
                    *e == ash::vk::KHR_BUFFER_DEVICE_ADDRESS_NAME
                        || *e == ash::vk::EXT_BUFFER_DEVICE_ADDRESS_NAME
                });
                if !has_bda && caps.supports_extension(ash::vk::EXT_BUFFER_DEVICE_ADDRESS_NAME) {
                    args.extensions
                        .push(ash::vk::EXT_BUFFER_DEVICE_ADDRESS_NAME);
                    let feats = Box::leak(Box::new(
                        ash::vk::PhysicalDeviceBufferDeviceAddressFeatures {
                            buffer_device_address: 1,
                            ..Default::default()
                        },
                    ));
                    feats.p_next = args.create_info.p_next as *mut c_void;
                    args.create_info.p_next = feats as *mut _ as *const c_void;
                }
            });
        }

        // Shared status, readable from both worlds.
        let status = DlssNeuralRenderingStatus(Arc::new(AtomicU8::new(STATUS_UNKNOWN)));
        app.insert_resource(status);

        // Mirror the camera entity (and later its settings) into the render world.
        app.add_plugins(SyncComponentPlugin::<DlssNeuralRendering>::default());
        app.add_systems(Update, ensure_camera_texture_usages);
        // `reset` is a one-frame request: it was extracted at the end of the
        // previous frame, clear it before this frame's systems run.
        app.add_systems(First, clear_reset_requests);
    }

    fn finish(&self, app: &mut App) {
        // finish() runs after every plugin's build(), so the RenderApp exists now.
        let status = app.world().resource::<DlssNeuralRenderingStatus>().clone();
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("bevy_dlss5: no RenderApp; is a render backend enabled?");
            status.set(STATUS_UNAVAILABLE);
            return;
        };
        render_app.insert_resource(Dlss5Runtime {
            inner: Mutex::new(Runtime {
                core: None,
                failed: false,
                source: self.dll_source.clone(),
                project_id: self.project_id.clone(),
                status,
                views: HashMap::new(),
                device: None,
            }),
        });
        render_app.add_systems(ExtractSchedule, extract_dlss_settings);
        render_app.add_systems(Render, shutdown_dlss_runtime.in_set(RenderSystems::Cleanup));
        // Must run after tonemapping: feature 18 wants display-referred color,
        // and the sRGB bridge clamps to [0, 1]. Without this constraint the
        // scheduler put the node before tonemapping, the clamp cut the HDR
        // frame at linear 1.0, and nothing on the NR side could get brighter
        // than TonyMcMapface(1.0) (198/255): emissives, sun patches and bloom
        // all lost their top end.
        render_app.add_systems(
            Core3d,
            dlss_nr_node
                .in_set(Core3dSystems::PostProcess)
                .after(tonemapping),
        );
    }
}

fn clear_reset_requests(mut cams: Query<&mut DlssNeuralRendering>) {
    for mut settings in &mut cams {
        if settings.reset {
            settings.reset = false;
        }
    }
}

/// Copy each camera's [`DlssNeuralRendering`] settings into the render world.
fn extract_dlss_settings(
    mut commands: Commands,
    cams: Extract<Query<(RenderEntity, &DlssNeuralRendering)>>,
    exits: Extract<Res<Messages<AppExit>>>,
) {
    if !exits.is_empty() {
        commands.insert_resource(Dlss5Shutdown);
    }
    for (render_entity, settings) in &cams {
        commands.entity(render_entity).insert(settings.clone());
    }
}

// ---------------------------------------------------------------------------
// Render-world runtime (lazily initialized; per-view feature handles)
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct Dlss5Shutdown;

#[derive(Resource)]
struct Dlss5Runtime {
    inner: Mutex<Runtime>,
}
struct Runtime {
    core: Option<NgxCore>,
    failed: bool,
    source: DllSource,
    project_id: String,
    status: DlssNeuralRenderingStatus,
    /// Keyed by the *main-world* camera entity (stable across frames).
    views: HashMap<Entity, ViewState>,
    // Keep Vulkan alive until features and the snippet library are released.
    device: Option<RenderDevice>,
}

impl Runtime {
    fn shutdown(&mut self) {
        let Some(core) = &self.core else {
            return;
        };
        if let Some(device) = &self.device {
            if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
                warn!("bevy_dlss5: waiting for GPU work during shutdown failed: {error}");
            }
        }
        let count = self.views.len();
        for (_, view) in self.views.drain() {
            if view.feature != 0 {
                // SAFETY: this core created the handle, the owning device is
                // retained, queued work has finished, and shutdown is exclusive.
                unsafe { core.release_feature(view.feature as *mut c_void) };
            }
        }
        // The bridge textures and feature handles must go before DLL unload.
        self.core = None;
        self.device = None;
        self.failed = true;
        self.status.set(STATUS_UNKNOWN);
        info!("bevy_dlss5: released {count} view(s) before runtime shutdown");
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Also cover an application dropping its App without an AppExit message.
        self.shutdown();
    }
}

fn shutdown_dlss_runtime(shutdown: Option<Res<Dlss5Shutdown>>, mut runtime: ResMut<Dlss5Runtime>) {
    if shutdown.is_none() {
        return;
    }
    // Cleanup runs after render submission, while other NGX users (including
    // Ray Reconstruction) and their SDK resources still exist in the world.
    match runtime.inner.get_mut() {
        Ok(inner) => inner.shutdown(),
        Err(poisoned) => poisoned.into_inner().shutdown(),
    }
}

struct ViewState {
    /// Feature-18 handle as usize so ViewState stays Send + Sync. 0 = failed.
    feature: usize,
    bridge: Bridge,
    width: u32,
    height: u32,
    evals: u64,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Feature 18 writes its output as a storage image, so a camera running neural
/// rendering must allow STORAGE_BINDING on its main texture and TEXTURE_BINDING
/// on its depth texture.
fn ensure_camera_texture_usages(
    mut cams: Query<(&mut CameraMainTextureUsages, &mut Camera3d), With<DlssNeuralRendering>>,
) {
    for (mut usages, mut cam3d) in &mut cams {
        usages.0 |= TextureUsages::STORAGE_BINDING;
        let mut d = TextureUsages::from(cam3d.depth_texture_usages);
        d |= TextureUsages::TEXTURE_BINDING;
        cam3d.depth_texture_usages = d.into();
    }
}

/// The neural-rendering post-process, run once per view that has a
/// [`DlssNeuralRendering`] component.
fn dlss_nr_node(
    view: ViewQuery<(
        &MainEntity,
        &DlssNeuralRendering,
        &ViewTarget,
        &ViewPrepassTextures,
    )>,
    device: Res<RenderDevice>,
    adapter: Res<RenderAdapter>,
    runtime: Res<Dlss5Runtime>,
    mut ctx: RenderContext,
) {
    let (main_entity, settings, view_target, prepass) = view.into_inner();
    let (Some(depth_pt), Some(mvec_pt)) = (&prepass.depth, &prepass.motion_vectors) else {
        return; // prepass textures not ready this frame
    };
    let key = main_entity.id();

    let mut guard = runtime.inner.lock().unwrap();
    if guard.failed {
        return;
    }

    // Lazy one-time NGX init, on the render thread with a live device.
    if guard.core.is_none() {
        let handles = unsafe { ngx::device_handles(device.wgpu_device()) };
        let result = match handles {
            None => Err("render device is not on the Vulkan backend".to_string()),
            Some(h) => {
                let name = adapter.get_info().name;
                let proj = guard.project_id.clone();
                match resolve_dll(&guard.source, &name) {
                    Ok(dll) => {
                        info!("bevy_dlss5: loading {} for {name}", dll.display());
                        unsafe { NgxCore::init(&proj, &dll, h) }
                    }
                    Err(e) => Err(e),
                }
            }
        };
        match result {
            Ok(core) => {
                info!("bevy_dlss5: DLSS 5 Neural Rendering initialized");
                guard.status.set(STATUS_ACTIVE);
                guard.device = Some((*device).clone());
                guard.core = Some(core);
            }
            Err(reason) => {
                warn!("bevy_dlss5: neural rendering unavailable: {reason}");
                guard.status.set(STATUS_UNAVAILABLE);
                guard.failed = true;
                return;
            }
        }
    }

    let wd = device.wgpu_device();
    let depth_view = &depth_pt.texture.default_view;
    let depth_tex = &depth_pt.texture.texture;
    let mvec_view = &mvec_pt.texture.default_view;
    let mvec_tex = &mvec_pt.texture.texture;
    let w = depth_tex.width();
    let h = depth_tex.height();

    // (Re)create per-view state on first sight or resolution change. Creating a
    // feature records weight uploads into its own command buffer; evaluate
    // starts next frame.
    let needs_create = guard
        .views
        .get(&key)
        .map(|v| v.width != w || v.height != h)
        .unwrap_or(true);
    if needs_create {
        if let Some(old) = guard.views.remove(&key) {
            if old.feature != 0 {
                let core = guard.core.as_ref().unwrap();
                unsafe { core.release_feature(old.feature as *mut c_void) };
            }
        }
        let dev_h = ngx::vk_handle(wd);
        let mut enc = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("dlss5_create"),
        });
        let cb = ngx::raw_command_buffer(&mut enc);
        let created = {
            let core = guard.core.as_ref().unwrap();
            unsafe { core.create_feature(dev_h, cb, w, h, settings.preset.code()) }
        };
        let feature = match created {
            Ok(f) => f as usize,
            Err(code) => {
                warn!("bevy_dlss5: CreateFeature(18) failed (0x{code:08X}); disabling this view");
                0
            }
        };
        let bridge = Bridge::new(wd, w, h);
        guard.views.insert(
            key,
            ViewState {
                feature,
                bridge,
                width: w,
                height: h,
                evals: 0,
            },
        );
        ctx.add_command_buffer(enc.finish());
        return;
    }

    // Split the guard borrow: shared core + one mutable view.
    let Runtime { core, views, .. } = &mut *guard;
    let core = core.as_ref().unwrap();
    let vs = views.get_mut(&key).unwrap();
    if vs.feature == 0 {
        return; // creation failed earlier for this view
    }
    // Temporal history is kept across frames (it accumulates detail and is
    // stable with Bevy's motion vectors); drop it on the first evaluation and
    // whenever the app asks (camera cuts).
    let reset = vs.evals == 0 || settings.reset;
    vs.evals += 1;
    let bridge = &vs.bridge;

    // Read and write the SAME main texture in place (no post_process_write
    // ping-pong, which would flicker every other frame).
    let main_view = view_target.main_texture_view();

    // Pass 1: encode linear -> sRGB into proxy_in.
    bridge.pass(
        wd,
        ctx.command_encoder(),
        false,
        main_view,
        &bridge.proxy_in_view,
        (w, h),
    );

    // Transition proxy + guides for the raw NGX evaluate.
    use wgpu::{TextureTransition, TextureUses};
    let dep_wt: &wgpu::Texture = depth_tex;
    let mv_wt: &wgpu::Texture = mvec_tex;
    ctx.command_encoder().transition_resources(
        core::iter::empty::<wgpu::BufferTransition<&wgpu::Buffer>>(),
        [
            TextureTransition {
                texture: &bridge.proxy_in,
                selector: None,
                state: TextureUses::RESOURCE,
            },
            TextureTransition {
                texture: dep_wt,
                selector: None,
                state: TextureUses::RESOURCE,
            },
            TextureTransition {
                texture: mv_wt,
                selector: None,
                state: TextureUses::RESOURCE,
            },
            TextureTransition {
                texture: &bridge.proxy_out,
                selector: None,
                state: TextureUses::STORAGE_READ_WRITE,
            },
        ]
        .into_iter(),
    );

    // Pass 2: feature 18 on the proxy (proxy_in -> proxy_out).
    let f16 = VK_FORMAT_R16G16B16A16_SFLOAT;
    let (mut color, mut output, mut depth, mut mvec) = unsafe {
        (
            ResourceVk::image_view(
                ngx::raw_view(&bridge.proxy_in_view),
                ngx::raw_image(&bridge.proxy_in),
                f16,
                w,
                h,
                false,
                false,
            ),
            ResourceVk::image_view(
                ngx::raw_view(&bridge.proxy_out_view),
                ngx::raw_image(&bridge.proxy_out),
                f16,
                w,
                h,
                false,
                true,
            ),
            ResourceVk::image_view(
                ngx::raw_view(depth_view),
                ngx::raw_image(depth_tex),
                ngx::vk_format(&adapter, depth_tex),
                w,
                h,
                true,
                false,
            ),
            ResourceVk::image_view(
                ngx::raw_view(mvec_view),
                ngx::raw_image(mvec_tex),
                ngx::vk_format(&adapter, mvec_tex),
                w,
                h,
                false,
                false,
            ),
        )
    };
    let mut enc = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("dlss5_eval"),
    });
    let cb = ngx::raw_command_buffer(&mut enc);
    let code = unsafe {
        core.evaluate(
            cb,
            vs.feature as *mut c_void,
            &mut color,
            &mut output,
            &mut depth,
            &mut mvec,
            w,
            h,
            settings.intensity,
            settings.tone,
            settings.structure,
            reset,
        )
    };
    ctx.add_command_buffer(enc.finish());

    // Pass 3: decode sRGB proxy -> linear back into the main texture.
    let mut dec = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("dlss5_decode"),
    });
    bridge.pass(
        wd,
        &mut dec,
        true,
        &bridge.proxy_out_view,
        main_view,
        (w, h),
    );
    ctx.add_command_buffer(dec.finish());

    use std::sync::atomic::AtomicBool;
    static LOGGED: AtomicBool = AtomicBool::new(false);
    if !LOGGED.swap(true, Ordering::Relaxed) {
        info!("bevy_dlss5: EvaluateFeature(18) -> 0x{code:08X}");
    }
}
