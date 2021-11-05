use pollster::block_on;
use wgpu::{Backends, DeviceDescriptor, Instance, RequestAdapterOptions};
use winit::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent}, window::WindowBuilder};

fn main() {
    let icon = winit::window::Icon::from_rgba(vec![0, 200, 0, 255].repeat(16*16), 16, 16);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Voxelcraft 0.0.1").with_window_icon(icon.ok()).build(&event_loop).unwrap();
    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: Default::default(),
            limits: Default::default(),
        },
        None,
    ))
    .unwrap();

    event_loop.run(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
