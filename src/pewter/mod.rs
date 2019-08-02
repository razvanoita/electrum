use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use crate::tin;

pub trait Buffer {
    fn new(device: &ash::Device, device_mem_prop: &vk::PhysicalDeviceMemoryProperties, size: vk::DeviceSize, usage: vk::BufferUsageFlags) -> Self;
    fn destroy(&self, device: &ash::Device);

    fn get_buffer(device: &ash::Device, size: vk::DeviceSize, usage: vk::BufferUsageFlags) -> vk::Buffer {
        unsafe {
            let buffer_info = vk::BufferCreateInfo {
                size: size,
                usage: usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };
            
            device.create_buffer(&buffer_info, None)
                .unwrap()
        }
    }

    fn get_device_memory(device: &ash::Device, buffer: &vk::Buffer, device_mem_prop: &vk::PhysicalDeviceMemoryProperties) -> vk::DeviceMemory {
        unsafe {
            let buffer_mem_req = device.get_buffer_memory_requirements(*buffer);
            let buffer_mem_idx = tin::find_memorytype_index(
                &buffer_mem_req, 
                &device_mem_prop, 
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            )
                .expect("Failed to find suitable memory type for buffer!");
            let buffer_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: buffer_mem_req.size,
                memory_type_index: buffer_mem_idx,
                ..Default::default()
            };

            device.allocate_memory(&buffer_allocate_info, None)
                .unwrap()
        }
    }
}

pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize
}

impl Buffer for VertexBuffer {
    fn new(device: &ash::Device, device_mem_prop: &vk::PhysicalDeviceMemoryProperties, size: vk::DeviceSize, usage: vk::BufferUsageFlags) -> Self {
        let buffer = Self::get_buffer(device, size, usage);
        VertexBuffer {
            buffer: buffer,
            memory: Self::get_device_memory(device, &buffer, device_mem_prop),
            size: size
        }
    }

    fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }
}

pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub count: usize,
    pub stride: usize
}

impl IndexBuffer {
    pub fn new(device: &ash::Device, device_mem_prop: &vk::PhysicalDeviceMemoryProperties, count: usize, stride: usize) -> Self {
        unsafe {
             let index_buffer_info = vk::BufferCreateInfo::builder()
                .size(count as u64 * stride as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let index_buffer = device.create_buffer(&index_buffer_info, None)
                .unwrap();
            let index_buffer_mem_req = device.get_buffer_memory_requirements(index_buffer);
            let index_buffer_mem_idx = tin::find_memorytype_index(
                &index_buffer_mem_req, 
                &device_mem_prop, 
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            )
                .expect("Failed to find memory type for index buffer!");
            let index_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: index_buffer_mem_req.size,
                memory_type_index: index_buffer_mem_idx,
                ..Default::default()
            };
            let index_buffer_mem = device.allocate_memory(&index_allocate_info, None)
                .unwrap();

            IndexBuffer {
                buffer: index_buffer,
                memory: index_buffer_mem,
                count: count,
                stride: stride
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

pub struct UniformBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub descriptor: vk::DescriptorBufferInfo
}

impl UniformBuffer {

}