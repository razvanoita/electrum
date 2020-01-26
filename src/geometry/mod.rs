use cgmath::*;

use crate::components;
use crate::render;
use crate::demo;

use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use std::mem;
use std::mem::align_of;

use render::buffer::Buffer;
use demo::end_and_submit_command_buffer;

pub mod platonic;

pub fn mesh(
    geometry: platonic::GeometryData, 
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
            std::mem::size_of::<platonic::Vertex>() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            false
        );
        let vb_staging_ptr = device.map_memory(vb_staging.memory, 0, vb_staging.size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut vb_staging_aligned_ptr = Align::new(vb_staging_ptr, align_of::<platonic::Vertex>() as u64, vb_staging.size);
        vb_staging_aligned_ptr.copy_from_slice(&geometry.vertices);
        device.unmap_memory(vb_staging.memory);
        device.bind_buffer_memory(vb_staging.buffer, vb_staging.memory, 0)
            .unwrap();

        let vb = render::buffer::VertexBuffer::construct(
            device, 
            mem_prop,
            geometry.vertices.len() as u64,
            std::mem::size_of::<platonic::Vertex>() as u64,
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
        let ib_staging_ptr = device.map_memory(ib_staging.memory, 0, ib_staging.count * ib_staging.stride, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut ib_staging_aligned_ptr = Align::new(ib_staging_ptr, align_of::<u32>() as u64, ib_staging.count * ib_staging.stride);
        ib_staging_aligned_ptr.copy_from_slice(&geometry.indices);
        device.unmap_memory(ib_staging.memory);
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
                .size(vb_staging.size)
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