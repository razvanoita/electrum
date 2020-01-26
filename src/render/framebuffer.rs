use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use crate::demo;

pub struct RenderTarget {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub format: vk::Format
}

impl RenderTarget {
    pub fn new(
        device: &ash::Device,
        device_mem_prop: &vk::PhysicalDeviceMemoryProperties,
        format: vk::Format, 
        usage: vk::ImageUsageFlags,
        width: u32,
        height: u32,
        depth: u32
    ) -> RenderTarget {
        unsafe {
            let mut aspect_mask = vk::ImageAspectFlags::empty();
            let mut image_layout: vk::ImageLayout;

            if usage.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
                aspect_mask = vk::ImageAspectFlags::COLOR;
                image_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            }
            if usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
                aspect_mask = vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL;
                image_layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
            }

            assert!(!aspect_mask.is_empty());

            let image_create_info = vk::ImageCreateInfo::builder()
                .usage(usage | vk::ImageUsageFlags::SAMPLED)
                .tiling(vk::ImageTiling::OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .array_layers(1)
                .mip_levels(1)
                .extent(vk::Extent3D {
                    width: width,
                    height: height,
                    depth: depth,
                })
                .format(format)
                .image_type(vk::ImageType::TYPE_2D)
                .build();

            let image = device.create_image(&image_create_info, None).unwrap();
            let mem_req = device.get_image_memory_requirements(image);

            let memory_type_index = demo::find_memorytype_index(&mem_req, device_mem_prop, vk::MemoryPropertyFlags::DEVICE_LOCAL).unwrap();

            let mem_alloc_info = vk::MemoryAllocateInfo::builder()
                .memory_type_index(memory_type_index)
                .allocation_size(mem_req.size)
                .build();

            let memory = device.allocate_memory(&mem_alloc_info, None).unwrap();
            device.bind_image_memory(image, memory, 0);

            let image_view_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .subresource_range(vk::ImageSubresourceRange::builder()
                    .layer_count(1)
                    .base_array_layer(0)
                    .level_count(1)
                    .base_mip_level(0)
                    .aspect_mask(aspect_mask)
                    .build()
                )
                .format(format)
                .view_type(vk::ImageViewType::TYPE_2D)
                .build();

            let view = device.create_image_view(&image_view_info, None).unwrap();
            
            RenderTarget {
                image: image,
                memory: memory,
                view: view,
                format: format
            }
        }
    }
}

pub struct Framebuffer {
    width: u32,
    height: u32,
    framebuffer: vk::Framebuffer,
    render_pass: vk::RenderPass,
    render_targets: Vec<RenderTarget>,
}