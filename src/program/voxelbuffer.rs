use std::{mem, ops::{Index, IndexMut}};

use bytemuck::{Pod, Zeroable};

pub struct VoxelBuffer {
    gpu_buffer: wgpu::Buffer,
    cpu_buffer: Vec<OctreeNode>,
    root: u32,
    root_level: u32,
    // TODO: allocation
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct OctreeNode {
    children: [u32; 8],
}

enum Octant {
    X0Y0Z0 = 0b000,
    X0Y0Z1 = 0b100,
    X0Y1Z0 = 0b010,
    X0Y1Z1 = 0b110,
    X1Y0Z0 = 0b001,
    X1Y0Z1 = 0b101,
    X1Y1Z0 = 0b011,
    X1Y1Z1 = 0b111,
}

impl Index<Octant> for OctreeNode {
    type Output = u32;
    fn index(&self, index: Octant) -> &Self::Output {
        &self.children[index as usize]
    }
}

impl IndexMut<Octant> for OctreeNode {
    fn index_mut(&mut self, index: Octant) -> &mut Self::Output {
        &mut self.children[index as usize]
    }
}

impl OctreeNode {
    const fn new() -> Self {
        Self {
            children: [0; 8],
        }
    }
}

impl VoxelBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut cpu_buffer = vec![OctreeNode::new(); 512];
        let root = 0;
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("voxel buffer descriptor"),
            size: mem::size_of::<OctreeNode>() as u64 * 512,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        cpu_buffer[root as usize][Octant::X1Y1Z1] = 0b11;
        cpu_buffer[root as usize][Octant::X1Y1Z0] = (1 << 2) | 0b01;
        // cpu_buffer[root as usize][Octant::X1Y0Z1] = 0b0;
        // cpu_buffer[root as usize][Octant::X1Y0Z0] = 0b0;
        // cpu_buffer[root as usize][Octant::X0Y1Z1] = 0b0;
        // cpu_buffer[root as usize][Octant::X0Y1Z0] = 0b0;
        // cpu_buffer[root as usize][Octant::X0Y0Z1] = 0b0;
        // cpu_buffer[root as usize][Octant::X0Y0Z0] = 0b0;

        
        // cpu_buffer[1][Octant::X1Y1Z1] = 0b0;
        // cpu_buffer[1][Octant::X1Y1Z0] = 0b0;
        // cpu_buffer[1][Octant::X1Y0Z1] = 0b0;
        cpu_buffer[1][Octant::X1Y0Z0] = 0b11;
        // cpu_buffer[1][Octant::X0Y1Z1] = 0b0;
        cpu_buffer[1][Octant::X0Y1Z0] = (2 << 2) | 0b01;
        // cpu_buffer[1][Octant::X0Y0Z1] = 0b0;
        // cpu_buffer[1][Octant::X0Y0Z0] = 0b0;

        cpu_buffer[2][Octant::X0Y1Z0] = 0b11;
        cpu_buffer[2][Octant::X0Y0Z1] = 0b11;

        cpu_buffer[2][Octant::X1Y0Z0] = (3 << 2) | 0b01;

        cpu_buffer[3][Octant::X0Y0Z0] = 0b11;


        Self {
            gpu_buffer,
            cpu_buffer,
            root,
            root_level: 0,
        }
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(self.cpu_buffer.as_slice()));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }
}
