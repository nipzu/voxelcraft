use std::time::Instant;

use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Program {
    window: Window,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    surface_config: wgpu::SurfaceConfiguration,
    frametimer: Instant,
    camera: Camera,
}

mod camera;

use camera::Camera;

impl Program {
    pub fn new() -> (EventLoop<()>, Self) {
        let icon = winit::window::Icon::from_rgba(vec![0, 150, 0, 255].repeat(16 * 16), 16, 16);
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Voxelcraft 0.0.1")
            .with_window_icon(icon.ok())
            .with_inner_size(PhysicalSize::new(1280_u32, 720))
            .with_min_inner_size(PhysicalSize::new(1_u32, 1))
            .with_transparent(true)
            .with_fullscreen(
                None, //Some(winit::window::Fullscreen::Borderless(None))
            )
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: Default::default(),
                limits: Default::default(),
            },
            None,
        ))
        .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        surface.configure(&device, &config);

        let shader_text = std::fs::read_to_string("shader.wgsl").unwrap();

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(shader_text.into()),
        });

        let (camera, camera_bind_group_layout) = Camera::new(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vert_main",
                buffers: &[],
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
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        window.set_cursor_visible(false);
        window.set_cursor_grab(true).unwrap();

        (
            event_loop,
            Self {
                window,
                surface,
                adapter,
                device,
                queue,
                pipeline,
                frametimer: Instant::now(),
                surface_config: config,
                camera,
            },
        )
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> ! {
        log::info!("{:?}", self.adapter.features());
        log::info!("{:?}", self.adapter.get_info());
        log::info!("{:?}", self.adapter.get_downlevel_properties());
        log::info!("{:?}", self.adapter.limits());

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            self.handle_event(event, control_flow);
        });
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            // self.surface.configure(&self.device, &self.surface_config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // log::info!(
        //     "rendering {:.3} fps",
        //     1_000_000.0 / self.frametimer.elapsed().as_micros() as f64
        // );
        self.frametimer = Instant::now();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        output.present();
        Ok(())
    }

    fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, window_id } if window_id == self.window.id() => {
                // log::info!("window event {:?}", event);
                self.handle_window_event(event, control_flow)
            }
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                // log::info!("redraw");

                if let Err(e) = self.render() {
                    log::error!("{:?}", e)
                }
            }
            Event::DeviceEvent { event, .. } => {
                self.handle_device_event(event);
            }
            Event::MainEventsCleared => {
                if let Err(e) = self.render() {
                    log::error!("{:?}", e)
                }
            }
            _ => (),
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                *control_flow = ControlFlow::Exit
            }
            WindowEvent::KeyboardInput { input, .. }
                if input.virtual_keycode == Some(VirtualKeyCode::Escape)
                    && input.state == ElementState::Pressed =>
            {
                *control_flow = ControlFlow::Exit
            }
            WindowEvent::Resized(new_size)
            | WindowEvent::ScaleFactorChanged {
                new_inner_size: &mut new_size,
                ..
            } => {
                self.resize(new_size);
            }
            _ => (),
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.camera.rotate(&self.queue, delta);
            }
            _ => (),
        }
    }
}
