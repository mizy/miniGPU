use std::collections::HashMap;

use bytemuck::{AnyBitPattern, NoUninit};
use wgpu::{util::DeviceExt, BufferUsages};
use winit::window::Window;

use crate::{scene::Scene, system::system::System, utils::depth_texture};

pub struct Renderer {
    pub config: RendererConfig,
    pub swapchain_format: wgpu::TextureFormat,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub window: Window,
    pub systems_map: HashMap<String, Box<dyn System>>,
    pub depth_texture: depth_texture::DepthTexture,
}

pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub async fn new(config: RendererConfig, window: Window) -> Renderer {
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: config.width,
            height: config.height,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);
        let depth_texture =
            depth_texture::DepthTexture::new(&device, &surface_config, "depth_texture");
        Renderer {
            window,
            config,
            surface_config,
            swapchain_format,
            adapter,
            surface,
            device,
            queue,
            depth_texture,
            systems_map: HashMap::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        self.depth_texture =
            depth_texture::DepthTexture::new(&self.device, &self.surface_config, "depth_texture");
    }

    pub fn render(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let map = &self.systems_map;
        for (_, system) in map {
            {
                system.update(self, scene);
            }
        }
        Ok(())
    }

    pub fn add_system(&mut self, name: String, system: Box<dyn System>) -> Option<Box<dyn System>> {
        self.systems_map.insert(name, system)
    }

    pub fn remove_system(&mut self, name: &str) {
        let system = self.systems_map.remove(name).unwrap();
        drop(system);
    }
}
