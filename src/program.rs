use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::Window,
};

mod camera;
mod controller;
mod framecounter;
mod voxelbuffer;

use camera::Camera;
use controller::CameraController;
use framecounter::FrameCounter;

use self::voxelbuffer::VoxelBuffer;

pub struct Program<'a> {
    render_ctx: Option<RenderCtx<'a>>,
    fps_counter: FrameCounter,
    controller: CameraController,
}

struct RenderCtx<'a> {
    window: Arc<Window>,
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    surface_config: wgpu::SurfaceConfiguration,
    bind_group: wgpu::BindGroup,
    camera: Camera,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Self {
            render_ctx: None,
            fps_counter: FrameCounter::new(0.5),
            controller: CameraController::default(),
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let render_ctx = self.render_ctx.as_mut().unwrap();
        if new_size.width > 0 && new_size.height > 0 {
            render_ctx.surface_config.width = new_size.width;
            render_ctx.surface_config.height = new_size.height;
            // self.surface.configure(&self.device, &self.surface_config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_ctx = self.render_ctx.as_mut().unwrap();
        let delta = self.fps_counter.new_frame();
        render_ctx.camera.transform(self.controller.cur_dir() * delta);

        if let Some(fps) = self.fps_counter.report() {
            let pos = render_ctx.camera.pos;
            log::info!(
                "fps: {fps:.1}, camera pos: {:.3} {:.3} {:.3}",
                pos.x,
                pos.y,
                pos.z
            );
        }
        let output = render_ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = render_ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&render_ctx.pipeline);
        render_ctx.camera.update_buffer(&render_ctx.queue);
        render_pass.set_bind_group(0, &render_ctx.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        render_ctx.queue.submit(Some(encoder.finish()));

        output.present();
        Ok(())
    }

    // fn handle_event(&mut self, event: Event<()>, control_flow: &ActiveEventLoop) {
    //     control_flow.set_control_flow(ControlFlow::Poll);
    //     match event {
    //         Event::WindowEvent { event, window_id } if window_id == self.window.id() => {
    //             // log::info!("window event {:?}", event);
    //             self.handle_window_event(event, control_flow)
    //         }
    //         Event::DeviceEvent { event, .. } => {
    //             self.handle_device_event(event);
    //         }
    //         // TODO: what to do about this?
    //         Event::AboutToWait => {
    //             if let Err(e) = self.render() {
    //                 log::error!("{:?}", e)
    //             }
    //         }
    //         _ => (),
    //     }
    // }
}

impl<'a> ApplicationHandler for Program<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let icon = winit::window::Icon::from_rgba([0, 150, 0, 255].repeat(16 * 16), 16, 16);
    let window_attrs = Window::default_attributes()
        .with_title("Voxelcraft 0.0.1")
        .with_window_icon(icon.ok())
        // .with_inner_size(PhysicalSize::new(640_u32, 360))
        .with_inner_size(PhysicalSize::new(1280_u32, 720))
        // .with_inner_size(PhysicalSize::new(1920_u32, 1080))
        // .with_inner_size(PhysicalSize::new(2560_u32, 1440))
        .with_min_inner_size(PhysicalSize::new(1_u32, 1))
        .with_transparent(false)
        .with_fullscreen(
            None,
            // Some(winit::window::Fullscreen::Borderless(None)),
        )
        .with_resizable(false);
        let window = Arc::new(event_loop.create_window(window_attrs).expect("unable to create window"));


        // TODO: only VULKAN
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("could not create surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let mut limits: wgpu::Limits = Default::default();
        // 0.5 GiB
        limits.max_buffer_size = 1 << 29;
        limits.max_storage_buffer_binding_size = 1 << 29;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: Default::default(),
                required_limits: limits,
                // TODO: which is more important
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .unwrap();

        let mut config = surface
            .get_default_config(
                &adapter,
                window.inner_size().width,
                window.inner_size().height,
            )
            .expect("could not get surface default config");
        config.present_mode = wgpu::PresentMode::AutoNoVsync;

        surface.configure(&device, &config);

        let shader_text = std::fs::read_to_string("shader.wgsl").unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(shader_text.into()),
        });

        let mut voxel_buffer = VoxelBuffer::new(&device);

        let camera = Camera::new(&device);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: voxel_buffer.buffer().as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vert_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "frag_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        window.set_cursor_visible(false);
        // TODO: wtf
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .expect("could not set cursor grab mode");

        camera.update_buffer(&queue);
        // TODO: only when needed
        voxel_buffer.update_buffer(&queue);

        log::info!("{:#?}", adapter.features());
        log::info!("{:#?}", adapter.get_info());
        log::info!("{:#?}", adapter.get_downlevel_capabilities());
        log::info!("{:#?}", adapter.limits());

        self.render_ctx = Some(RenderCtx {
                window,
                bind_group,
                surface,
                adapter,
                device,
                queue,
                pipeline,
                camera,
                surface_config: config,
            });
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.render_ctx.as_ref().unwrap().window.request_redraw()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                // log::info!("redraw");

                if let Err(e) = self.render() {
                    log::error!("{:?}", e)
                }
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Escape)
                    && event.state == ElementState::Pressed =>
            {
                event_loop.exit()
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.controller.handle_key_event(event);
            }
            // TODO: scale factor
            WindowEvent::Resized(new_size) => {
                self.resize(new_size);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.render_ctx.as_mut().unwrap().camera.rotate(delta);
            }
            _ => (),
        }
    }
}
