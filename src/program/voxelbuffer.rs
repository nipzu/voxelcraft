use std::{mem, ops::{Index, IndexMut}};

use bytemuck::{Pod, Zeroable};
use nalgebra::Vector3;

pub struct VoxelBuffer {
    gpu_buffer: wgpu::Buffer,
    cpu_buffer: Vec<OctreeNode>,
    root: usize,
    root_level: u32,
    // TODO: allocation
    bump_top: usize,
    uploaded: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct OctreeNode {
    children: [u32; 8],
}

#[derive(Debug, Clone, Copy)]
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
        let mut cpu_buffer = vec![OctreeNode::new(); 30_000_000];
        let root = 0;
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("voxel buffer descriptor"),
            size: (mem::size_of::<OctreeNode>() * cpu_buffer.len()) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // cpu_buffer[root][Octant::X1Y1Z1] = 0b11;
        // cpu_buffer[root][Octant::X1Y1Z0] = (1 << 2) | 0b01;
        // // cpu_buffer[root as usize][Octant::X1Y0Z1] = 0b0;
        // // cpu_buffer[root as usize][Octant::X1Y0Z0] = 0b0;
        // // cpu_buffer[root as usize][Octant::X0Y1Z1] = 0b0;
        // // cpu_buffer[root as usize][Octant::X0Y1Z0] = 0b0;
        // // cpu_buffer[root as usize][Octant::X0Y0Z1] = 0b0;
        // // cpu_buffer[root as usize][Octant::X0Y0Z0] = 0b0;

        
        // // cpu_buffer[1][Octant::X1Y1Z1] = 0b0;
        // cpu_buffer[1][Octant::X1Y1Z0] = (3 << 2) | 0b01;
        // // cpu_buffer[1][Octant::X1Y0Z1] = 0b0;
        // cpu_buffer[1][Octant::X1Y0Z0] = 0b11;
        // // cpu_buffer[1][Octant::X0Y1Z1] = 0b0;
        // cpu_buffer[1][Octant::X0Y1Z0] = (2 << 2) | 0b01;
        // // cpu_buffer[1][Octant::X0Y0Z1] = 0b0;
        // // cpu_buffer[1][Octant::X0Y0Z0] = 0b0;

        // cpu_buffer[2][Octant::X0Y1Z0] = 0b11;
        // cpu_buffer[2][Octant::X0Y0Z1] = 0b11;

        // cpu_buffer[2][Octant::X1Y0Z0] = (3 << 2) | 0b01;

        // cpu_buffer[3][Octant::X0Y0Z0] = 0b11;

        // cpu_buffer[4][Octant::X1Y0Z0] = 0b11;


        let mut s = Self {
            gpu_buffer,
            cpu_buffer,
            root,
            root_level: 0,
            bump_top: 0,
            uploaded: false,
        };

        
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
        for _ in 0..1_000 {
            let x = (rng.gen_range(0..=0b1111_1111) << 23) + (0b0100_0000 << 24);
            let y = (rng.gen_range(0..=0b1111_1111) << 23) + (0b0100_0000 << 24);
            let z = (rng.gen_range(0..=0b1111_1111) << 23) + (0b0100_0000 << 24);
            s.add_voxel(Vector3::new(x,y,z), 6);
        }


        let k = 11;
        for x in (0b01 << k)..(0b11 << k) {
            for z in (0b01 << k)..(0b11 << k) {
                let xs = 2.0 * f64::from(x - (0b01 << k)) / f64::from(0b10 << k) - 1.0;
                let zs = 2.0 * f64::from(z - (0b01 << k)) / f64::from(0b10 << k) - 1.0;
                // let ys = 0.5 * (2.0 - xs*xs - zs*zs);
                let ys = 0.5 * (1.0 - xs*zs);
                s.add_voxel(Vector3::new(x << (30 - k), ((ys * f64::from(0b10 << k) + f64::from(0b01 << k)) as u32) << (30 - k), z << (30 - k)), k + 1);
            }   
        }

        // s.add_voxel(Vector3::new(0,0,0), 3);
        // s.add_voxel(Vector3::new(0b1010_0000 << 24,0b1010_0000 << 24,0b1010_0000 << 24), 1);
        // s.add_voxel(Vector3::new(0b1100_0000 << 24,0b0100_0000 << 24,0b0100_0000 << 24), 1);
        // s.add_voxel(Vector3::new(0b0110_0000 << 24,0b1010_0000 << 24,0b0100_0000 << 24), 3);
        // s.add_voxel(Vector3::new(0,0,0), 2);

        s
    }

    fn add_voxel(&mut self, mut pos: Vector3<u32>, size: u32) {
        let mut cur_ocnode_idx = self.root as usize;
        let top_bit = 1 << 31;
        for _ in 0..size {
            let idx = match (pos.x & top_bit != 0, pos.y & top_bit != 0, pos.z & top_bit != 0) {
                (false, false, false) => Octant::X0Y0Z0,
                (false, false, true)  => Octant::X0Y0Z1,
                (false, true, false) =>  Octant::X0Y1Z0,
                (false, true, true)  =>  Octant::X0Y1Z1,
                (true, false, false) =>  Octant::X1Y0Z0,
                (true, false, true) =>   Octant::X1Y0Z1,
                (true, true, false) =>   Octant::X1Y1Z0,
                (true, true, true) =>    Octant::X1Y1Z1,
            };
            if self.cpu_buffer[cur_ocnode_idx][idx] == 0 {
                self.bump_top += 1;
                self.cpu_buffer[cur_ocnode_idx][idx] = 4*(self.bump_top as u32) + 0b01;
            } else if self.cpu_buffer[cur_ocnode_idx][idx] == 0b11 {
                return;
                panic!();
            }
            cur_ocnode_idx = self.cpu_buffer[cur_ocnode_idx][idx] as usize >> 2;
            pos.x <<= 1;
            pos.y <<= 1;
            pos.z <<= 1;
        }
        let idx = match (pos.x & top_bit != 0, pos.y & top_bit != 0, pos.z & top_bit != 0) {
            (false, false, false) => Octant::X0Y0Z0,
            (false, false, true)  => Octant::X0Y0Z1,
            (false, true, false) =>  Octant::X0Y1Z0,
            (false, true, true)  =>  Octant::X0Y1Z1,
            (true, false, false) =>  Octant::X1Y0Z0,
            (true, false, true) =>   Octant::X1Y0Z1,
            (true, true, false) =>   Octant::X1Y1Z0,
            (true, true, true) =>    Octant::X1Y1Z1,
        };
        self.cpu_buffer[cur_ocnode_idx][idx] = 0b11; 
    }

    pub fn update_buffer(&mut self, queue: &wgpu::Queue) {
        if !self.uploaded {
            queue.write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(self.cpu_buffer.as_slice()));
            self.uploaded = true;
            println!("uploading...");
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }
}
