use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use crate::demo;

use std::mem::align_of;

pub trait Buffer {
    fn construct(
        device: &ash::Device, 
        mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        count: u64, 
        stride: u64, 
        usage: vk::BufferUsageFlags,
        mem_prop_flags: vk::MemoryPropertyFlags,
        dynamic: bool
    ) -> Self;

    fn destroy(&self, device: &ash::Device);

    fn construct_buffer(device: &ash::Device, size: u64, usage: vk::BufferUsageFlags) -> vk::Buffer {
        unsafe {
            let buffer_info = vk::BufferCreateInfo {
                size: size,
                usage: usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };
            
            device.create_buffer(&buffer_info, None)
                .expect("Faile to create buffer!")
        }
    }

    fn allocate_memory(
        device: &ash::Device, 
        buffer: &vk::Buffer, 
        device_mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        mem_prop_flags: vk::MemoryPropertyFlags
    ) -> vk::DeviceMemory {
        unsafe {
            let buffer_mem_req = device.get_buffer_memory_requirements(*buffer);

            let buffer_mem_idx = demo::find_memorytype_index(
                &buffer_mem_req, 
                &device_mem_prop, 
                mem_prop_flags
            )
                .expect("Failed to find suitable memory type for buffer!");
                
            let buffer_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: buffer_mem_req.size,
                memory_type_index: buffer_mem_idx,
                ..Default::default()
            };

            device.allocate_memory(&buffer_allocate_info, None)
                .expect("Failed to allocate memory for buffer!")
        }
    }
}

pub fn copy_to_buffer<T: Copy>(
    device: &ash::Device, 
    memory: vk::DeviceMemory,
    data: &[T]
) {
    unsafe {
        let size = (std::mem::size_of::<T>() * data.len()) as u64;
        let ptr = device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut aligned_ptr = Align::new(ptr, align_of::<T>() as u64, size);
        aligned_ptr.copy_from_slice(&data);
        device.unmap_memory(memory);
    }
}

// ---

pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub count: u64,
    pub stride: u64
}

impl Buffer for VertexBuffer {
    fn construct(
        device: &ash::Device, 
        mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        count: u64, 
        stride: u64, 
        usage: vk::BufferUsageFlags,
        mem_prop_flags: vk::MemoryPropertyFlags,
        dynamic: bool
    ) -> Self {
        let buffer = Self::construct_buffer(device, count * stride, usage);
        VertexBuffer {
            buffer: buffer,
            memory: Self::allocate_memory(device, &buffer, mem_prop, mem_prop_flags),
            count: count,
            stride: stride,
        }
    }

    fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }
}
// ---

pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub count: u64,
    pub stride: u64
}

impl Buffer for IndexBuffer {
    fn construct(
        device: &ash::Device, 
        mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        count: u64, 
        stride: u64, 
        usage: vk::BufferUsageFlags,
        mem_prop_flags: vk::MemoryPropertyFlags,
        dynamic: bool
    ) -> Self {
        let buffer = Self::construct_buffer(device, count * stride, usage);
        IndexBuffer {
            buffer: buffer,
            memory: Self::allocate_memory(device, &buffer, mem_prop, mem_prop_flags),
            count: count,
            stride: stride
        }
    }

    fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }
}

// ---

pub struct UniformBuffer {
    pub memory: vk::DeviceMemory,
    pub descriptor: vk::DescriptorBufferInfo,
    pub size: u64,
}

impl Buffer for UniformBuffer {
    fn construct(
        device: &ash::Device, 
        mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        count: u64, 
        stride: u64, 
        usage: vk::BufferUsageFlags,
        mem_prop_flags: vk::MemoryPropertyFlags,
        dynamic: bool
    ) -> Self {
        let buffer = Self::construct_buffer(device, count * stride, usage);
        let desc_buff_info = vk::DescriptorBufferInfo {
            buffer: buffer,
            offset: 0 as u64,
            range: if let false = dynamic { stride * count } else { stride }
        };
        UniformBuffer {
            memory: Self::allocate_memory(device, &desc_buff_info.buffer, mem_prop, mem_prop_flags),
            descriptor: desc_buff_info,
            size: count * stride
        }
    }

    fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.descriptor.buffer, None);
        }
    }
}

// ---

pub struct RayTracingBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl Buffer for RayTracingBuffer {
    fn construct(
        device: &ash::Device, 
        mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        count: u64, 
        stride: u64, 
        usage: vk::BufferUsageFlags,
        mem_prop_flags: vk::MemoryPropertyFlags,
        dynamic: bool
    ) -> Self {
        let buffer = Self::construct_buffer(device, count * stride, usage);
        RayTracingBuffer {
            buffer: buffer,
            memory: Self::allocate_memory(device, &buffer, mem_prop, mem_prop_flags),
        }
    }

    fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.free_memory(self.memory, None);
            device.destroy_buffer(self.buffer, None);
        }
    }
}

impl RayTracingBuffer {
    pub fn new(
        device: &ash::Device, 
        mem_prop: &vk::PhysicalDeviceMemoryProperties, 
        count: u64, 
        stride: u64,
        mem_prop_flags: vk::MemoryPropertyFlags,
    ) -> Self {
        Buffer::construct(
            &device,
            &mem_prop,
            count,
            stride,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            mem_prop_flags,
            false
        )
    }
}