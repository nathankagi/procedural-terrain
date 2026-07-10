use std::os::raw;
use std::sync::Arc;
use std::time::Instant;

use super::camera;
use super::model;
use super::model::{DrawLight, Vertex};
use super::pipeline::create_render_pipeline;
use super::scene::{self, MaterialHandle, ModelHandle, Object, PipelineHandle};
use super::texture;
use super::transform::{Transform, TransformRaw};
use crate::assets;
use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

const GUI_UPDATE_RATE: f32 = 1.0 / 2.0;
const LIGHT_ORBIT_DEG_PER_SEC: f32 = 15.0;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    colour: [f32; 3],
    _padding2: u32,
}

struct CameraState {
    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: camera::CameraController,
}

struct LightState {
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    light_model: model::Model,
}

struct RenderState {
    render_pipelines: Vec<wgpu::RenderPipeline>,
    depth_texture: texture::Texture,
    instance_buffer: wgpu::Buffer,
    instance_capacity: u32,
    models: Vec<model::Model>,
    materials: Vec<model::Material>,
}

struct GuiState {
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    last_update_time: Instant,
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    pub(crate) window: Arc<Window>,
    scene: scene::Scene,
    camera: CameraState,
    render: RenderState,
    light: LightState,
    gui_state: GuiState,
    last_frame_time: Instant,
    last_cursor_pos: Option<(f64, f64)>,
    fps: f32,
}

fn create_instance_buffer(device: &wgpu::Device, capacity: u32) -> wgpu::Buffer {
    let capacity = capacity.max(1) as u64;
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Instance Buffer"),
        size: capacity * std::mem::size_of::<TransformRaw>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn sync_instance_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    capacity: &mut u32,
    instances: &[TransformRaw],
) {
    if instances.is_empty() {
        return;
    }

    let required = instances.len() as u32;
    if required > *capacity {
        let new_capacity = required.max(capacity.saturating_mul(2));
        *buffer = create_instance_buffer(device, new_capacity);
        *capacity = new_capacity;
    }
    queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
}

impl State {
    pub(crate) async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|&mode| mode == wgpu::PresentMode::Fifo)
            .unwrap_or(surface_caps.present_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera = camera::Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 100.0,
            znear: 0.1,
            zfar: 500.0,
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera_controller = camera::CameraController::Orbit(camera::OrbitCamera::new());

        let light_uniform = LightUniform {
            position: [20.0, 3.0, 5.0],
            _padding: 0,
            colour: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&light_bind_group_layout),
                ],
                immediate_size: 0,
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        let mut materials: Vec<model::Material> = Vec::new();

        let light_model = assets::load_obj_model(
            &"cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
            &mut materials,
        )
        .await
        .unwrap();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&camera_bind_group_layout),
                    Some(&light_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
            };

            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), TransformRaw::desc()],
                shader,
            )
        };

        let render_pipelines = vec![render_pipeline];

        // let size = 25;
        // let params = heightmaps::perlin::FractalPerlinParams {
        //     height: size,
        //     width: size,
        //     scale: 20.0,
        //     octaves: 12,
        //     persistence: 0.65,
        //     seed: 10,
        // };

        // let heightmap = heightmaps::lib::HeightMap::generate(
        //     heightmaps::lib::Algorithms::FractalPerlin(params),
        // );

        // let meshed = heightmap.to_mesh();

        // let model_name = "heightmap".to_string();
        // let model = resources::model_from_mesh(
        //     &model_name,
        //     &device,
        //     &queue,
        //     &texture_bind_group_layout,
        //     meshed,
        //     &mut materials,
        // )
        // .await
        // .unwrap();

        let terrain_model = assets::load_heightmap_png(
            "test_map.png",
            &device,
            &queue,
            &texture_bind_group_layout,
            &mut materials,
        )
        .await
        .unwrap();

        let mut models = Vec::new();

        let terrain_handle = ModelHandle(models.len());
        models.push(terrain_model);

        let mut scene = scene::Scene::new();
        scene.spawn(Object {
            model: terrain_handle,
            pipeline: PipelineHandle(0),
            material: None,
            transform: Transform::identity(),
        });

        let mut instance_buffer = create_instance_buffer(&device, 0);
        let mut instance_capacity = 0u32;
        {
            let (batched, _) = scene.batches();
            sync_instance_buffer(
                &device,
                &queue,
                &mut instance_buffer,
                &mut instance_capacity,
                &batched.instances,
            );
        }

        let camera_state = CameraState {
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
        };

        let light_state = LightState {
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            light_model,
        };

        let render_state = RenderState {
            render_pipelines,
            depth_texture,
            instance_buffer,
            instance_capacity,
            models,
            materials,
        };

        let egui_ctx = egui::Context::default();

        let gui_state = GuiState {
            egui_ctx: egui_ctx.clone(),
            egui_state: egui_winit::State::new(
                egui_ctx.clone(),
                egui::ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                None,
                None,
            ),
            egui_renderer: egui_wgpu::Renderer::new(
                &device,
                config.format,
                egui_wgpu::RendererOptions::default(),
            ),
            last_update_time: Instant::now(),
        };

        let mut state = Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            scene,
            camera: camera_state,
            render: render_state,
            light: light_state,
            gui_state: gui_state,
            last_frame_time: Instant::now(),
            last_cursor_pos: None,
            fps: 0.0,
        };

        state.resize(size.width, size.height);
        state.window.request_redraw();

        Ok(state)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.render.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.camera.camera.aspect = width as f32 / height as f32;
        }
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        let gui_dt = now
            .duration_since(self.gui_state.last_update_time)
            .as_secs_f32();
        if gui_dt > GUI_UPDATE_RATE {
            self.gui_state.last_update_time = now;

            self.fps = 1.0 / dt;
        }

        self.camera
            .camera_controller
            .update(&mut self.camera.camera, dt);
        self.camera
            .camera_uniform
            .update_view_proj(&self.camera.camera);
        self.queue.write_buffer(
            &self.camera.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.camera_uniform]),
        );

        let old_position: cgmath::Vector3<_> = self.light.light_uniform.position.into();
        self.light.light_uniform.position = (cgmath::Quaternion::from_axis_angle(
            (0.0, 1.0, 0.0).into(),
            cgmath::Deg(LIGHT_ORBIT_DEG_PER_SEC * dt),
        ) * old_position)
            .into();
        self.queue.write_buffer(
            &self.light.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light.light_uniform]),
        );

        // handle event
    }

    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        let response = self
            .gui_state
            .egui_state
            .on_window_event(&self.window, event);
        response.consumed
    }

    pub(crate) fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture();
        let surface_texture = match output {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(()),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Validation => {
                return Err(anyhow::anyhow!("Surface texture unavailable"));
            }
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let (batched, changed) = self.scene.batches();
        if changed {
            sync_instance_buffer(
                &self.device,
                &self.queue,
                &mut self.render.instance_buffer,
                &mut self.render.instance_capacity,
                &batched.instances,
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.light.light_render_pipeline);
            render_pass.draw_light_model(
                &self.light.light_model,
                &self.camera.camera_bind_group,
                &self.light.light_bind_group,
            );

            render_pass.set_vertex_buffer(1, self.render.instance_buffer.slice(..));
            render_pass.set_bind_group(1, &self.camera.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light.light_bind_group, &[]);

            let mut last_pipeline: Option<PipelineHandle> = None;
            let mut last_material: Option<MaterialHandle> = None;

            for batch in &batched.batches {
                if last_pipeline != Some(batch.pipeline) {
                    render_pass.set_pipeline(&self.render.render_pipelines[batch.pipeline.0]);
                    last_pipeline = Some(batch.pipeline);
                }

                let model = &self.render.models[batch.model.0];
                for mesh in &model.meshes {
                    let material_handle = batch.material.unwrap_or(mesh.material);
                    if last_material != Some(material_handle) {
                        let material = &self.render.materials[material_handle.0];
                        render_pass.set_bind_group(0, &material.bind_group, &[]);
                        last_material = Some(material_handle);
                    }

                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.num_elements, 0, batch.instance_range.clone());
                }
            }
        }

        let raw_input = self.gui_state.egui_state.take_egui_input(&self.window);
        let egui_output = self.gui_state.egui_ctx.run_ui(raw_input, |ctx| {
            egui::Window::new("Debug")
                .default_pos((10.0, 10.0))
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("FPS: {:.2}", self.fps));
                });
        });

        self.gui_state
            .egui_state
            .handle_platform_output(&self.window, egui_output.platform_output);
        let tris = self
            .gui_state
            .egui_ctx
            .tessellate(egui_output.shapes, egui_output.pixels_per_point);
        for (id, image_delta) in &egui_output.textures_delta.set {
            self.gui_state.egui_renderer.update_texture(
                &self.device,
                &self.queue,
                *id,
                image_delta,
            );
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: egui_output.pixels_per_point,
        };

        self.gui_state.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.gui_state.egui_renderer.render(
                &mut egui_pass.forget_lifetime(),
                &tris,
                &screen_descriptor,
            );
        }

        for id in &egui_output.textures_delta.free {
            self.gui_state.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }

    pub(crate) fn handle_key(
        &mut self,
        event_loop: &ActiveEventLoop,
        code: KeyCode,
        is_pressed: bool,
    ) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera.camera_controller.handle_key(code, is_pressed);
        }
    }

    pub(crate) fn handle_mouse_button(&mut self, button: MouseButton, is_pressed: bool) -> bool {
        self.camera
            .camera_controller
            .handle_mouse_button(button, is_pressed)
    }

    pub(crate) fn handle_mouse_motion(&mut self, x: f64, y: f64) {
        if let Some((last_x, last_y)) = self.last_cursor_pos {
            self.camera.camera_controller.handle_mouse_motion(
                &mut self.camera.camera,
                x - last_x,
                y - last_y,
            );
        }
        self.last_cursor_pos = Some((x, y));
    }

    pub(crate) fn handle_scroll(&mut self, delta: f32) {
        self.camera
            .camera_controller
            .handle_scroll(&mut self.camera.camera, delta);
    }
}
