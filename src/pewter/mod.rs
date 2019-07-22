use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use crate::tin;

pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize
}

impl VertexBuffer {
    pub fn new(device: &ash::Device, device_mem_prop: &vk::PhysicalDeviceMemoryProperties, size: vk::DeviceSize) -> Self {
        unsafe {
            let vertex_buffer_info = vk::BufferCreateInfo {
                size: size,
                usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };

            let vertex_buffer = device.create_buffer(&vertex_buffer_info, None)
                .unwrap();
            let vertex_buffer_mem_req = device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_buffer_mem_idx = tin::find_memorytype_index(
                &vertex_buffer_mem_req, 
                &device_mem_prop, 
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            )
                .expect("Failed to find suitable memory type for vertex buffer!");
            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: vertex_buffer_mem_req.size,
                memory_type_index: vertex_buffer_mem_idx,
                ..Default::default()
            };
            let vertex_buffer_mem = device.allocate_memory(&vertex_buffer_allocate_info, None)
                .unwrap();

            VertexBuffer {
                buffer: vertex_buffer,
                memory: vertex_buffer_mem,
                size: vertex_buffer_mem_req.size
            }
        }
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }
}