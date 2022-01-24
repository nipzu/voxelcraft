use nalgebra::{Rotation3, Vector3};

use std::f64::consts::{FRAC_PI_2, PI};

pub struct Camera {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    /// radians between -pi/2 and pi/2
    pitch: f64,
    /// radians between 0 and 2pi
    yaw: f64,
}

impl Camera {
    pub fn new(device: &wgpu::Device) -> (Self, wgpu::BindGroupLayout) {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera canvas buffer"),
            size: 16 * std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera buffer layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        (
            Self {
                bind_group,
                buffer,
                pitch: 0.0,
                yaw: 0.0,
            },
            bind_group_layout,
        )
    }

    pub fn rotate(&mut self, queue: &wgpu::Queue, mut delta: (f64, f64)) {
        delta.0 /= 1000.0;
        delta.1 /= 1000.0;

        self.pitch = f64::clamp(self.pitch + delta.1, -FRAC_PI_2, FRAC_PI_2);
        self.yaw = (self.yaw + delta.0) % (2.0 * PI);

        let pitch_rot = Rotation3::from_scaled_axis(self.pitch * Vector3::x());
        let yaw_rot = Rotation3::from_scaled_axis(self.yaw * Vector3::y());
        let rot = yaw_rot * pitch_rot;

        let dir = rot * Vector3::z();
        let r = yaw_rot * Vector3::x() * 1280.0 / 1000.0;
        let u = rot * Vector3::y() * 720.0 / 1000.0;

        let data = CameraData {
            origin: Vec3F32 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                pad: 0.0,
            },
            canvas_mid_delta: Vec3F32 {
                x: dir.x as f32,
                y: dir.y as f32,
                z: dir.z as f32,
                pad: 0.0,
            },
            canvas_top_delta: Vec3F32 {
                x: u.x as f32,
                y: u.y as f32,
                z: u.z as f32,
                pad: 0.0,
            },
            canvas_right_delta: Vec3F32 {
                x: r.x as f32,
                y: r.y as f32,
                z: r.z as f32,
                pad: 0.0,
            },
        };

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&data));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
struct Vec3F32 {
    x: f32,
    y: f32,
    z: f32,
    pad: f32,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(64))]
struct CameraData {
    origin: Vec3F32,
    canvas_mid_delta: Vec3F32,
    canvas_top_delta: Vec3F32,
    canvas_right_delta: Vec3F32,
}
