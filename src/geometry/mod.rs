use cgmath::*;

use crate::components;
use crate::render;
use crate::demo;

use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use std::mem;

use render::buffer::Buffer;
use render::buffer::copy_to_buffer;
use demo::end_and_submit_command_buffer;

pub mod platonic;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3]
}

pub struct GeometryData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct RayTracingInstance {
    pub transform: [f32; 12],
    pub id_and_mask: u32,
    pub offset_and_flags: u32,
    pub as_handle: u64,
}

impl RayTracingInstance {
    pub fn new(
        transform: [f32; 12],
        id: u32,
        mask: u8,
        offset: u32,
        flags: vk::GeometryInstanceFlagsNV,
        handle: u64
    ) -> Self {
        let mut id_and_mask: u32 = 0;
        id_and_mask |= id & 0x00FFFFFF;
        id_and_mask |= (mask as u32) << 24;

        let mut offset_and_flags: u32 = 0;
        offset_and_flags |= offset & 0x00FFFFFF;
        offset_and_flags |= (flags.as_raw() as u32) << 24;

        RayTracingInstance {
            transform: transform,
            id_and_mask: id_and_mask,
            offset_and_flags: offset_and_flags,
            as_handle: handle,
        }
    }
}

pub fn mesh(
    geometry: GeometryData, 
    device: &ash::Device, 
    mem_prop: &vk::PhysicalDeviceMemoryProperties, 
    copy_command_buffer: vk::CommandBuffer,
    present_queue: vk::Queue
) -> components::Mesh {
    unsafe {
        let vb_staging = render::buffer::VertexBuffer::construct(
            device, 
            mem_prop,
            geometry.vertices.len() as u64,
            std::mem::size_of::<Vertex>() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            false
        );

        copy_to_buffer(&device, vb_staging.memory, &geometry.vertices);
        
        device.bind_buffer_memory(vb_staging.buffer, vb_staging.memory, 0)
            .unwrap();

        let vb = render::buffer::VertexBuffer::construct(
            device, 
            mem_prop,
            geometry.vertices.len() as u64,
            std::mem::size_of::<Vertex>() as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            false
        );
        device.bind_buffer_memory(vb.buffer, vb.memory, 0)
            .unwrap();

        let ib_staging = render::buffer::IndexBuffer::construct(
            device, 
            mem_prop,  
            geometry.indices.len() as u64,
            mem::size_of::<u32>() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            false
        );

        copy_to_buffer(&device, ib_staging.memory, &geometry.indices);

        device.bind_buffer_memory(ib_staging.buffer, ib_staging.memory, 0)
            .unwrap();

        let ib = render::buffer::IndexBuffer::construct(
            device, 
            mem_prop,  
            geometry.indices.len() as u64,
            mem::size_of::<u32>() as u64,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            false
        );
        device.bind_buffer_memory(ib.buffer, ib.memory, 0)
            .unwrap();

        let copy_region_vb = vk::BufferCopy::builder()
                .size(vb_staging.count * vb_staging.stride)
                .build();
        device.cmd_copy_buffer(copy_command_buffer, vb_staging.buffer, vb.buffer, &[copy_region_vb]);
        let copy_region_ib = vk::BufferCopy::builder()
            .size(ib_staging.count * ib_staging.stride)
            .build();
        device.cmd_copy_buffer(copy_command_buffer, ib_staging.buffer, ib.buffer, &[copy_region_ib]);

        end_and_submit_command_buffer(device, present_queue, copy_command_buffer);

        vb_staging.destroy(device);        
        ib_staging.destroy(device);

        components::Mesh {
            vertex_buffer: vb,
            index_buffer: ib
        }
    }
}

pub fn quad() -> GeometryData {
    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let vertex_data = vec![
        cgmath::Vector3 { x: 1.0, y: 1.0, z: 0.0 },
        cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0, },
        cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0, },
        cgmath::Vector3 { x: 1.0, y: 0.0, z: 0.0, }
    ];

    let faces = vec![
        [0, 1, 2],
        [2, 3, 0],
    ];

    let face_colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0]
    ];

    for (i, face) in faces.iter().enumerate() {
        let normal = cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        
        let base = data.vertices.len() as u32;
        data.indices.push(base);
        data.indices.push(base + 1);
        data.indices.push(base + 2); 

        data.vertices.push(
            Vertex {
                position: [vertex_data[face[0] as usize].x, vertex_data[face[0] as usize].y, vertex_data[face[0] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize] 
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[1] as usize].x, vertex_data[face[1] as usize].y, vertex_data[face[1] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[2] as usize].x, vertex_data[face[2] as usize].y, vertex_data[face[2] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
    }

    data
}