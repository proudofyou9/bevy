use std::ops::{Deref, DerefMut};

use crate::{
    render_resource::TextureView,
    renderer::{RenderDevice, RenderInstance},
    texture::BevyDefault,
    RenderStage,
};
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_utils::HashMap;
use bevy_window::{RawWindowHandleWrapper, WindowId, Windows};
use wgpu::{SwapChainFrame, TextureFormat};

pub struct WindowRenderPlugin;

impl Plugin for WindowRenderPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(0);
        render_app
            .init_resource::<WindowSurfaces>()
            .add_system_to_stage(RenderStage::Extract, extract_windows.system())
            .add_system_to_stage(RenderStage::Prepare, prepare_windows.system());
    }
}

pub struct ExtractedWindow {
    pub id: WindowId,
    pub handle: RawWindowHandleWrapper,
    pub physical_width: u32,
    pub physical_height: u32,
    pub vsync: bool,
    pub swap_chain_frame: Option<TextureView>,
}

#[derive(Default)]
pub struct ExtractedWindows {
    pub windows: HashMap<WindowId, ExtractedWindow>,
}

impl Deref for ExtractedWindows {
    type Target = HashMap<WindowId, ExtractedWindow>;

    fn deref(&self) -> &Self::Target {
        &self.windows
    }
}

impl DerefMut for ExtractedWindows {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.windows
    }
}

fn extract_windows(mut commands: Commands, windows: Res<Windows>) {
    let mut extracted_windows = ExtractedWindows::default();
    for window in windows.iter() {
        extracted_windows.insert(
            window.id(),
            ExtractedWindow {
                id: window.id(),
                handle: window.raw_window_handle(),
                physical_width: window.physical_width(),
                physical_height: window.physical_height(),
                vsync: window.vsync(),
                swap_chain_frame: None,
            },
        );
    }

    commands.insert_resource(extracted_windows);
}

#[derive(Default)]
pub struct WindowSurfaces {
    surfaces: HashMap<WindowId, wgpu::Surface>,
    swap_chains: HashMap<WindowId, wgpu::SwapChain>,
}

pub struct WindowSwapChain {
    value: TextureView,
}

pub fn prepare_windows(
    mut windows: ResMut<ExtractedWindows>,
    mut window_surfaces: ResMut<WindowSurfaces>,
    render_device: Res<RenderDevice>,
    render_instance: Res<RenderInstance>,
) {
    let window_surfaces = window_surfaces.deref_mut();
    for window in windows.windows.values_mut() {
        let surface = window_surfaces
            .surfaces
            .entry(window.id)
            .or_insert_with(|| unsafe {
                render_instance.create_surface(&window.handle.get_handle())
            });

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            format: TextureFormat::bevy_default(),
            width: window.physical_width,
            height: window.physical_height,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            present_mode: if window.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Immediate
            },
        };

        let swap_chain = window_surfaces
            .swap_chains
            .entry(window.id)
            .or_insert_with(|| render_device.create_swap_chain(surface, &swap_chain_descriptor));

        let frame = if let Ok(swap_chain_frame) = swap_chain.get_current_frame() {
            swap_chain_frame
        } else {
            let swap_chain = window_surfaces
                .swap_chains
                .entry(window.id)
                .or_insert_with(|| {
                    render_device.create_swap_chain(surface, &swap_chain_descriptor)
                });

            swap_chain
                .get_current_frame()
                .expect("Failed to acquire next swap chain texture!")
        };

        window.swap_chain_frame = Some(TextureView::from(frame));
    }
}
