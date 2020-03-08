use ash::util::*;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::{Device, Entry, Instance};

use cgmath::*;

mod components;
mod geometry;
mod world;
mod render;
mod demo;

use render::buffer::Buffer;

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use rand::Rng;
use std::default::Default;
use std::ffi::CString;
use std::mem;
use std::mem::align_of;
use std::ops::Mul;
use std::os::raw::c_void;

#[derive(Debug, Clone, Copy)]
struct ViewData {
    projection: cgmath::Matrix4<f32>,
    view: cgmath::Matrix4<f32>,
}

fn update_viewdata_uniform_buffer(
    mapped_memory: *mut c_void,
    alignment: vk::DeviceSize,
    size: vk::DeviceSize,
    projection_matrix: cgmath::Matrix4<f32>,
    view_matrix: cgmath::Matrix4<f32>
) {
    let view_data = [ViewData {
        projection: projection_matrix,
        view: view_matrix,
    }];
    unsafe {
        // --- copy data to host mapped memory
        let mut aligned_mapped_memory = Align::new(mapped_memory, alignment, size);
        aligned_mapped_memory.copy_from_slice(&view_data);
    }
}

fn update_dynamic_uniform_buffer(
    mapped_memory: *mut c_void,
    alignment: vk::DeviceSize,
    size: vk::DeviceSize,
    memory: vk::DeviceMemory,
    device: &ash::Device,
    instance_data: Vec<cgmath::Matrix4<f32>>,
) {
    unsafe {
        let mut aligned_mapped_memory = Align::new(mapped_memory, alignment, size);
        aligned_mapped_memory.copy_from_slice(&instance_data);

        let memory_range = vk::MappedMemoryRange {
            memory: memory,
            size: size,
            ..Default::default()
        };
        device.flush_mapped_memory_ranges(&[memory_range]);
    }
}

fn create_gbuffer(
    device: &ash::Device,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    width: u32, 
    height: u32
) -> render::framebuffer::Framebuffer {
    unsafe {
        let mut render_targets: Vec<render::framebuffer::RenderTarget> = Vec::new();

        render_targets.push(render::framebuffer::RenderTarget::new(
            &device,
            &device_memory_properties,
            vk::Format::A2R10G10B10_UNORM_PACK32,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            width,
            height,
            1,
        ));

        render_targets.push(render::framebuffer::RenderTarget::new(
            &device,
            &device_memory_properties,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            width,
            height,
            1,
        ));

        render_targets.push(render::framebuffer::RenderTarget::new(
            &device,
            &device_memory_properties,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            width,
            height,
            1,
        ));

        render_targets.push(render::framebuffer::RenderTarget::new(
            &device,
            &device_memory_properties,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            width,
            height,
            1,
        ));

        render_targets.push(render::framebuffer::RenderTarget::new(
            &device,
            &device_memory_properties,
            vk::Format::D24_UNORM_S8_UINT,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            width,
            height,
            1,
        ));

        let attachments_descs: Vec<vk::AttachmentDescription> = render_targets
            .iter()
            .map(|rt| {
                vk::AttachmentDescription::builder()
                    .format(rt.format)
                    .final_layout(if rt.format == vk::Format::D24_UNORM_S8_UINT {
                        vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
                    } else {
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                    })
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .build()
            })
            .collect();

        let mut color_refs: Vec<vk::AttachmentReference> = Vec::new();
        color_refs.push(vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        });
        color_refs.push(vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        });
        color_refs.push(vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        });
        color_refs.push(vk::AttachmentReference {
            attachment: 3,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        });

        let depth_ref = vk::AttachmentReference {
            attachment: 4,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass_desc = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_refs.as_slice())
            .depth_stencil_attachment(&depth_ref)
            .build();

        let subpass_dependencies: [vk::SubpassDependency; 2] = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::MEMORY_READ)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build()
        ];
        
        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments_descs)
            .subpasses(&[subpass_desc])
            .dependencies(&subpass_dependencies)
            .build();

        let renderpass = device.create_render_pass(&renderpass_info, None).unwrap();

        let image_views: [vk::ImageView; 5] = [
            render_targets[0].view,
            render_targets[1].view,
            render_targets[2].view,
            render_targets[3].view,
            render_targets[4].view,
        ];

        let fb_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(&image_views)
            .width(width)
            .height(height)
            .layers(1)
            .build();
        let fb = device.create_framebuffer(&fb_create_info, None).unwrap();

       render::framebuffer::Framebuffer {
            width: width,
            height: height,
            framebuffer: fb,
            render_pass: renderpass,
            render_targets: render_targets,
        }
    }
}

fn create_samplers(device: &ash::Device) -> (vk::Sampler, vk::Sampler) {
    unsafe {
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .max_anisotropy(1.0)
            .min_lod(0.0)
            .max_lod(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .build();

        (device.create_sampler(&sampler_info, None).unwrap(), device.create_sampler(&sampler_info, None).unwrap())
    }
}

fn main() {
    let mut world = world::World::new();
    unsafe {
        let demo_app = demo::DemoApp::new(1920, 1080);
        let mut demo = demo_app.build_ctx();

        let renderpass_atachments = [
            vk::AttachmentDescription {
                format: demo.surface_format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];

        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_atachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let renderpass = demo
            .device
            .create_render_pass(&renderpass_create_info, None)
            .unwrap();

        let framebuffers: Vec<vk::Framebuffer> = demo
            .present_image_views
            .iter()
            .map(|&view| {
                let fb_attachments = [view, demo.depth_image_view];
                let fb_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&fb_attachments)
                    .width(demo.surface_resolution.width)
                    .height(demo.surface_resolution.height)
                    .layers(1);

                demo.device
                    .create_framebuffer(&fb_create_info, None)
                    .unwrap()
            })
            .collect();

        // --- create shaders
        demo.add_shader("copper/shaders/bin/gbuffer_vert.spv");
        demo.add_shader("copper/shaders/bin/gbuffer_frag.spv");
        demo.add_shader("copper/shaders/bin/deferred_vert.spv");
        demo.add_shader("copper/shaders/bin/deferred_frag.spv");

        // --- prepare for deferred
        let gbuffer = create_gbuffer(&demo.device, &demo.device_memory_properties, demo.surface_resolution.width, demo.surface_resolution.height);
        let (color_sampler, depth_sampler) = create_samplers(&demo.device);

        let gbuffer_color_blend_attachment_states = vec![vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::all(),
        }; 3];
        let deferred_color_blend_attachment_states = vec![vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::all(),
        }; 1];

        let gbuffer_descriptor_set_layout = demo.create_descriptor_set_layout(
            vec![
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    binding: 0,
                    ..Default::default()
                },
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::VERTEX,
                    binding: 1,
                    ..Default::default()
                },
            ]
        );
        let gbuffer_pipeline_layout = demo.create_pipeline_layout(gbuffer_descriptor_set_layout);

        // --- gbuffer
        let deferred_descriptor_set_layout = demo.create_descriptor_set_layout(
            vec![
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    binding: 0,
                    ..Default::default()
                },
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    binding: 1,
                    ..Default::default()
                },
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    binding: 2,
                    ..Default::default()
                },
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    binding: 3,
                    ..Default::default()
                },
                vk::DescriptorSetLayoutBinding {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    binding: 4,
                    ..Default::default()
                },
            ]
        );
        let deferred_pipeline_layout = demo.create_pipeline_layout(deferred_descriptor_set_layout);

        // --- create platonic solids
        let copy_command_buffer_0 = demo.get_and_begin_command_buffer();
        let tetrahedron_mesh = geometry::mesh(
            geometry::platonic::tetrahedron(),
            &demo.device,
            &demo.device_memory_properties,
            copy_command_buffer_0,
            demo.present_queue,
        );
        let tetrahedron = world
            .create_entity()
            .with_component(components::Component::TransformComponent(
                components::Transform {
                    position: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    rotation: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            ))
            .with_component(components::Component::MeshComponent(tetrahedron_mesh))
            .with_component(components::Component::VelocityComponent(
                components::Velocity {
                    translation_speed: 1.0,
                    rotation_speed: 1.0,
                },
            ))
            .with_component(components::Component::MaterialComponent(
                components::Material {
                    vertex_shader: demo.get_shader_module("copper/shaders/bin/gbuffer_vert.spv"),
                    fragment_shader: demo.get_shader_module("copper/shaders/bin/gbuffer_frag.spv"),
                    pso: vk::Pipeline::null(),
                    render_pass: gbuffer.render_pass,
                    pipeline_layout: gbuffer_pipeline_layout,
                    color_blend_attachment_states: gbuffer_color_blend_attachment_states.clone(),
                },
            ))
            .build();

        let copy_command_buffer_1 = demo.get_and_begin_command_buffer();
        let cube_mesh = geometry::mesh(
            geometry::platonic::cube(),
            &demo.device,
            &demo.device_memory_properties,
            copy_command_buffer_1,
            demo.present_queue,
        );
        let cube = world
            .create_entity()
            .with_component(components::Component::TransformComponent(
                components::Transform {
                    position: cgmath::Vector3 {
                        x: 3.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    rotation: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            ))
            .with_component(components::Component::MeshComponent(cube_mesh))
            .with_component(components::Component::VelocityComponent(
                components::Velocity {
                    translation_speed: 1.0,
                    rotation_speed: 1.0,
                },
            ))
            .with_component(components::Component::MaterialComponent(
                components::Material {
                    vertex_shader: demo.get_shader_module("copper/shaders/bin/gbuffer_vert.spv"),
                    fragment_shader: demo.get_shader_module("copper/shaders/bin/gbuffer_frag.spv"),
                    pso: vk::Pipeline::null(),
                    render_pass: gbuffer.render_pass,
                    pipeline_layout: gbuffer_pipeline_layout,
                    color_blend_attachment_states: gbuffer_color_blend_attachment_states.clone(),
                },
            ))
            .build();

        let copy_command_buffer_2 = demo.get_and_begin_command_buffer();
        let octahedron_mesh = geometry::mesh(
            geometry::platonic::octahedron(),
            &demo.device,
            &demo.device_memory_properties,
            copy_command_buffer_2,
            demo.present_queue,
        );
        let octahedron = world
            .create_entity()
            .with_component(components::Component::TransformComponent(
                components::Transform {
                    position: cgmath::Vector3 {
                        x: -3.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    rotation: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            ))
            .with_component(components::Component::MeshComponent(octahedron_mesh))
            .with_component(components::Component::VelocityComponent(
                components::Velocity {
                    translation_speed: 1.0,
                    rotation_speed: 2.0,
                },
            ))
            .with_component(components::Component::MaterialComponent(
                components::Material {
                    vertex_shader: demo.get_shader_module("copper/shaders/bin/gbuffer_vert.spv"),
                    fragment_shader: demo.get_shader_module("copper/shaders/bin/gbuffer_frag.spv"),
                    pso: vk::Pipeline::null(),
                    render_pass: gbuffer.render_pass,
                    pipeline_layout: gbuffer_pipeline_layout,
                    color_blend_attachment_states: gbuffer_color_blend_attachment_states.clone(),
                },
            ))
            .build();

        let copy_command_buffer_3 = demo.get_and_begin_command_buffer();
        let dodecahedron_mesh = geometry::mesh(
            geometry::platonic::dodecahedron(),
            &demo.device,
            &demo.device_memory_properties,
            copy_command_buffer_3,
            demo.present_queue,
        );
        let dodecahedron = world
            .create_entity()
            .with_component(components::Component::TransformComponent(
                components::Transform {
                    position: cgmath::Vector3 {
                        x: 0.0,
                        y: -3.0,
                        z: 0.0,
                    },
                    rotation: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            ))
            .with_component(components::Component::MeshComponent(dodecahedron_mesh))
            .with_component(components::Component::VelocityComponent(
                components::Velocity {
                    translation_speed: 1.0,
                    rotation_speed: 1.5,
                },
            ))
            .with_component(components::Component::MaterialComponent(
                components::Material {
                    vertex_shader: demo
                        .get_shader_module("copper/shaders/bin/gbuffer_vert.spv"),
                    fragment_shader: demo
                        .get_shader_module("copper/shaders/bin/gbuffer_frag.spv"),
                    pso: vk::Pipeline::null(),
                    render_pass: gbuffer.render_pass,
                    pipeline_layout: gbuffer_pipeline_layout,
                    color_blend_attachment_states: gbuffer_color_blend_attachment_states.clone(),
                },
            ))
            .build();

        let copy_command_buffer_5 = demo.get_and_begin_command_buffer();
        let icosahedron_mesh = geometry::mesh(
            geometry::platonic::icosahedron(),
            &demo.device,
            &demo.device_memory_properties,
            copy_command_buffer_5,
            demo.present_queue,
        );
        let icosahedron = world
            .create_entity()
            .with_component(components::Component::TransformComponent(
                components::Transform {
                    position: cgmath::Vector3 {
                        x: 0.0,
                        y: 3.0,
                        z: 0.0,
                    },
                    rotation: cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: cgmath::Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
            ))
            .with_component(components::Component::MeshComponent(icosahedron_mesh))
            .with_component(components::Component::VelocityComponent(
                components::Velocity {
                    translation_speed: 1.0,
                    rotation_speed: 3.5,
                },
            ))
            .with_component(components::Component::MaterialComponent(
                components::Material {
                    vertex_shader: demo
                        .get_shader_module("copper/shaders/bin/gbuffer_vert.spv"),
                    fragment_shader: demo
                        .get_shader_module("copper/shaders/bin/gbuffer_frag.spv"),
                    pso: vk::Pipeline::null(),
                    render_pass: gbuffer.render_pass,
                    pipeline_layout: gbuffer_pipeline_layout,
                    color_blend_attachment_states: gbuffer_color_blend_attachment_states.clone(),
                },
            ))
            .build();

        let deferred_light = world
            .create_entity()
            .with_component(components::Component::MaterialComponent(
                components::Material {
                    vertex_shader: demo
                        .get_shader_module("copper/shaders/bin/deferred_vert.spv"),
                    fragment_shader: demo
                        .get_shader_module("copper/shaders/bin/deferred_frag.spv"),
                    pso: vk::Pipeline::null(),
                    render_pass: renderpass,
                    pipeline_layout: deferred_pipeline_layout,
                    color_blend_attachment_states: deferred_color_blend_attachment_states.clone(),
                },
            ))
            .build();

        // --- create dynamic uniform buffer
        let min_ub_alignment = demo
            .instance
            .get_physical_device_properties(demo.physical_device)
            .limits
            .min_uniform_buffer_offset_alignment;
        let mut dynamic_alignment = std::mem::size_of::<cgmath::Matrix4<f32>>() as u64;
        if (min_ub_alignment > 0) {
            dynamic_alignment =
                (dynamic_alignment + min_ub_alignment - 1) & !(min_ub_alignment - 1);
        }
        let ub_instance_data = render::buffer::UniformBuffer::construct(
            &demo.device,
            &demo.device_memory_properties,
            5 as u64,
            dynamic_alignment,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            true,
        );
        demo.device
            .bind_buffer_memory(
                ub_instance_data.descriptor.buffer,
                ub_instance_data.memory,
                0,
            )
            .unwrap();

        let ub_instance_data_ptr = demo
            .device
            .map_memory(
                ub_instance_data.memory,
                0,
                ub_instance_data.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();
        let mut rng = rand::thread_rng();
        let transform_filter = components::ComponentType::TransformComponent as u32;
        world
            .transform_storage
            .iter_mut()
            .filter(|entry| entry.storage_type & transform_filter == transform_filter)
            .for_each(|entry| {
                entry.component.rotation = cgmath::Vector3 {
                    x: rng.gen_range(-1.0, 1.0),
                    y: rng.gen_range(-1.0, 1.0),
                    z: rng.gen_range(-1.0, 1.0),
                } * 2.0 * 3.14;
            });

        // --- create non-dynamic uniform buffer
        let ub_view_data = render::buffer::UniformBuffer::construct(
            &demo.device,
            &demo.device_memory_properties,
            1,
            mem::size_of::<ViewData>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            false,
        );
        demo.device
            .bind_buffer_memory(ub_view_data.descriptor.buffer, ub_view_data.memory, 0)
            .unwrap();

        let ub_view_data_ptr = demo
            .device
            .map_memory(
                ub_view_data.memory,
                0,
                ub_view_data.descriptor.range,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap();
        update_viewdata_uniform_buffer(
            ub_view_data_ptr,
            mem::size_of::<ViewData>() as u64,
            ub_view_data.descriptor.range,
            cgmath::perspective(
                cgmath::Rad::from(cgmath::Deg(60.0)),
                demo.surface_resolution.width as f32 / demo.surface_resolution.height as f32,
                0.1,
                256.0,
            ),
            cgmath::Matrix4::look_at(
                cgmath::Point3::new(0.0, 0.0, -10.0),
                cgmath::Point3::new(0.0, 0.0, 5.0),
                cgmath::Vector3::new(0.0, 1.0, 0.0),
            )
        );

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: demo.surface_resolution.width as f32,
            height: demo.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: demo.surface_resolution.clone(),
        }];

        let ccb = demo.get_and_begin_command_buffer();
        let fullscreen_quad = geometry::mesh(
            geometry::quad(),
            &demo.device,
            &demo.device_memory_properties,
            ccb,
            demo.present_queue,
        );

        // --- build PSOs for objects
        let material_filter = components::ComponentType::MaterialComponent as u32;
        let mesh_filter = components::ComponentType::MeshComponent as u32;
        world
            .material_storage
            .iter_mut()
            .filter(|entry| entry.storage_type & material_filter == material_filter)
            .for_each(|entry| {
                entry.component.pso = demo.create_pso(
                    entry.component.vertex_shader,
                    entry.component.fragment_shader,
                    entry.component.render_pass,
                    entry.component.pipeline_layout,
                    viewports,
                    scissors,
                    &entry.component.color_blend_attachment_states,
                    if entry.storage_type & mesh_filter == mesh_filter {
                        demo::PSOCreateOption::HasVertexAttributes
                    } else {
                        demo::PSOCreateOption::NoVertexAttributes
                    },
                );
            });

        // --- setup descriptor pool
        let descriptor_pool_sizes = vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 5,
            },
        ];
        demo.create_descriptor_pool(descriptor_pool_sizes);

        // --- setup descriptor sets
        let deferred_descriptor_set_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[deferred_descriptor_set_layout])
            .descriptor_pool(demo.descriptor_pool)
            .build();
        let deferred_descriptor_sets = demo.device.allocate_descriptor_sets(&deferred_descriptor_set_info).unwrap();

        let gbuffer_info_0 = vk::DescriptorImageInfo::builder()
            .sampler(color_sampler)
            .image_view(gbuffer.render_targets[0].view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build();
        let gbuffer_info_1 = vk::DescriptorImageInfo::builder()
            .sampler(color_sampler)
            .image_view(gbuffer.render_targets[1].view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build();
        let gbuffer_info_2 = vk::DescriptorImageInfo::builder()
            .sampler(color_sampler)
            .image_view(gbuffer.render_targets[2].view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build();
        let gbuffer_info_3 = vk::DescriptorImageInfo::builder()
            .sampler(color_sampler)
            .image_view(gbuffer.render_targets[3].view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build();
        let gbuffer_info_4 = vk::DescriptorImageInfo::builder()
            .sampler(depth_sampler)
            .image_view(gbuffer.render_targets[4].view)
            .image_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
            .build();

        let deferred_write_descriptor_sets = [
            vk::WriteDescriptorSet::builder()
                .dst_binding(0)
                .image_info(&[gbuffer_info_0])
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(deferred_descriptor_sets[0])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_binding(1)
                .image_info(&[gbuffer_info_1])
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(deferred_descriptor_sets[0])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_binding(2)
                .image_info(&[gbuffer_info_2])
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(deferred_descriptor_sets[0])
                .build(),
                vk::WriteDescriptorSet::builder()
                .dst_binding(3)
                .image_info(&[gbuffer_info_3])
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(deferred_descriptor_sets[0])
                .build(),
                vk::WriteDescriptorSet::builder()
                .dst_binding(4)
                .image_info(&[gbuffer_info_4])
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .dst_set(deferred_descriptor_sets[0])
                .build(),
        ];

        demo.device.update_descriptor_sets(&deferred_write_descriptor_sets, &[]);

        let gbuffer_descriptor_set_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[gbuffer_descriptor_set_layout])
            .descriptor_pool(demo.descriptor_pool)
            .build();
        let gbuffer_descriptor_sets = demo.device.allocate_descriptor_sets(&gbuffer_descriptor_set_info).unwrap();

        let gbuffer_write_descriptor_sets = [
            vk::WriteDescriptorSet::builder()
                .dst_binding(0)
                .buffer_info(&[ub_view_data.descriptor])
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(gbuffer_descriptor_sets[0])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_binding(1)
                .buffer_info(&[ub_instance_data.descriptor])
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .dst_set(gbuffer_descriptor_sets[0])
                .build(),
        ];

        demo.device.update_descriptor_sets(&gbuffer_write_descriptor_sets, &[]);

        // --- create gbuffer command buffer
        let gbuffer_command_buffer = demo.get_command_buffer();

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let gbuffer_semaphore = demo.device.create_semaphore(&semaphore_create_info, None).unwrap();      

        let dt: f32 = 1.0 / 60.0;
        let mut current_time = std::time::SystemTime::now();
        let mut accumulator: f32 = 0.0;

        let shader_asset_bin_path: String = String::from("copper/shaders/bin");
        demo.watcher
            .watch(shader_asset_bin_path.clone(), RecursiveMode::Recursive)
            .unwrap();

        demo_app.run(|| {
            let asset_key = demo.process_asset_event();
            demo.receive_asset_event();

            if asset_key.is_some() {
                let key = asset_key.unwrap();
                let (old_shader_module, new_shader_module) = demo.reload_shader_module(&key);

                world
                    .material_storage
                    .iter_mut()
                    .filter(|entry| {
                        (entry.storage_type & material_filter == material_filter)
                            && (entry.component.vertex_shader == old_shader_module
                                || entry.component.fragment_shader == old_shader_module)
                    })
                    .for_each(|entry| {
                        if entry.component.vertex_shader == old_shader_module {
                            entry.component.vertex_shader = new_shader_module;
                        } else if entry.component.fragment_shader == old_shader_module {
                            entry.component.fragment_shader = new_shader_module;
                        }

                        demo.device.destroy_pipeline(entry.component.pso, None);
                        entry.component.pso = demo.create_pso(
                            entry.component.vertex_shader,
                            entry.component.fragment_shader,
                            entry.component.render_pass,
                            entry.component.pipeline_layout,
                            viewports,
                            scissors,
                            &entry.component.color_blend_attachment_states,
                            if entry.storage_type & mesh_filter == mesh_filter {
                                demo::PSOCreateOption::HasVertexAttributes
                            } else {
                                demo::PSOCreateOption::NoVertexAttributes
                            }
                        );
                    });

                demo.device.destroy_shader_module(old_shader_module, None);
            }

            let deferred_pso = world.material_storage
                .iter_mut()
                .find(|entry| entry.entity == deferred_light)
                .unwrap()
                .component
                .pso;

            let new_time = std::time::SystemTime::now();
            let frame_time =
                new_time.duration_since(current_time).unwrap().as_millis() as f32 / 1000.0;
            current_time = new_time;

            accumulator += frame_time;

            while accumulator >= dt {
                let transform_velocity_filter = (components::ComponentType::TransformComponent
                    as u32)
                    | (components::ComponentType::VelocityComponent as u32);
                world
                    .velocity_storage
                    .iter()
                    .filter(|entry| {
                        entry.storage_type & transform_velocity_filter == transform_velocity_filter
                    })
                    .zip(world.transform_storage.iter_mut().filter(|entry| {
                        entry.storage_type & transform_velocity_filter == transform_velocity_filter
                    }))
                    .for_each(|(velocity, transform)| {
                        transform.component.rotation += cgmath::Vector3 {
                            x: velocity.component.rotation_speed * dt,
                            y: velocity.component.rotation_speed * dt,
                            z: velocity.component.rotation_speed * dt,
                        };
                    });

                let instance_data: Vec<cgmath::Matrix4<f32>> = world
                    .transform_storage
                    .iter()
                    .filter(|entry| entry.storage_type & transform_filter == transform_filter)
                    .map(|entry| {
                        cgmath::Matrix4::from_translation(entry.component.position)
                            .mul(cgmath::Matrix4::from_axis_angle(
                                cgmath::Vector3 {
                                    x: 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                },
                                cgmath::Rad(entry.component.rotation.x),
                            ))
                            .mul(cgmath::Matrix4::from_axis_angle(
                                cgmath::Vector3 {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                },
                                cgmath::Rad(entry.component.rotation.y),
                            ))
                            .mul(cgmath::Matrix4::from_axis_angle(
                                cgmath::Vector3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 1.0,
                                },
                                cgmath::Rad(entry.component.rotation.z),
                            ))
                    })
                    .collect();

                update_dynamic_uniform_buffer(
                    ub_instance_data_ptr,
                    dynamic_alignment,
                    ub_instance_data.descriptor.range * instance_data.len() as u64,
                    ub_instance_data.memory,
                    &demo.device,
                    instance_data,
                );

                accumulator -= dt;
            }

            // --- we have done updates, record gbuffer command buffer
            demo::record_command_buffer(
                &demo.device,
                gbuffer_command_buffer,
                |device, draw_command_buffer| {
                    let clear_values = [
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
                            },
                        },
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
                            },
                        },
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
                            },
                        },
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 0.0],
                            },
                        },
                        vk::ClearValue {
                            depth_stencil: vk::ClearDepthStencilValue {
                                depth: 1.0,
                                stencil: 0,
                            },
                        },
                    ];
    
                    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                        .render_pass(gbuffer.render_pass)
                        .framebuffer(gbuffer.framebuffer)
                        .render_area(vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent: demo.surface_resolution.clone(),
                        })
                        .clear_values(&clear_values);
    
                    device.cmd_begin_render_pass(draw_command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    let mut dynamic_offset = 0;
    
                    let mesh_material_filter = (components::ComponentType::MeshComponent as u32)
                        | (components::ComponentType::MaterialComponent as u32);
                    world
                        .mesh_storage
                        .iter()
                        .filter(|entry| {
                            entry.storage_type & mesh_material_filter == mesh_material_filter
                        })
                        .zip(world.material_storage.iter().filter(|entry| {
                            entry.storage_type & mesh_material_filter == mesh_material_filter
                        }))
                        .for_each(|(mesh, material)| {
                            device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, material.component.pso);
    
                            device.cmd_bind_descriptor_sets(
                                draw_command_buffer,
                                vk::PipelineBindPoint::GRAPHICS,
                                gbuffer_pipeline_layout,
                                0,
                                &gbuffer_descriptor_sets,
                                &[dynamic_offset * dynamic_alignment as u32],
                            );
    
                            device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[mesh.component.vertex_buffer.buffer], &[0]);
                            device.cmd_bind_index_buffer(draw_command_buffer, mesh.component.index_buffer.buffer, 0, vk::IndexType::UINT32);
                            device.cmd_draw_indexed(draw_command_buffer, mesh.component.index_buffer.count as u32, 1, 0, 0, 1);
                            dynamic_offset += 1;
                        });
    
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );

            // --- now record deferred command buffers
            for i in 0..demo.draw_command_buffers.len() {
                demo::record_command_buffer(
                    &demo.device,
                    demo.draw_command_buffers[i],
                    |device, draw_command_buffer| {
                        let clear_values = [
                            vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [0.0, 0.0, 0.0, 0.0],
                                },
                            },
                            vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            },
                        ];
        
                        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                            .render_pass(renderpass)
                            .framebuffer(framebuffers[i as usize])
                            .render_area(vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: demo.surface_resolution.clone(),
                            })
                            .clear_values(&clear_values);
        
                        device.cmd_begin_render_pass(draw_command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                        device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                        device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
        
                        device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, deferred_pso);
                        device.cmd_bind_descriptor_sets(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, deferred_pipeline_layout, 0, &deferred_descriptor_sets, &[0]);
                        device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[fullscreen_quad.vertex_buffer.buffer], &[0]);
                        device.cmd_bind_index_buffer(draw_command_buffer, fullscreen_quad.index_buffer.buffer, 0, vk::IndexType::UINT32);
                        device.cmd_draw_indexed(draw_command_buffer, fullscreen_quad.index_buffer.count as u32, 1, 0, 0, 1);
        
                        device.cmd_end_render_pass(draw_command_buffer);
                    }
                );
            }

            let (present_idx, _) = demo.swapchain_loader.acquire_next_image(demo.swapchain, std::u64::MAX, demo.present_complete_semaphore, vk::Fence::null())
                .unwrap();

            // --- wait for swapchain present to finish, submit gbuffer command buffer and signal gbuffer semaphore 
            let gbuffer_submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[demo.present_complete_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[gbuffer_command_buffer])
                .signal_semaphores(&[gbuffer_semaphore])
                .build();

            demo.device.queue_submit(demo.present_queue, &[gbuffer_submit_info], vk::Fence::null())
                .expect("Failed to submit queue!");

            // --- wait for gbuffer semaphore, submit deferred command buffer and signal render complete semaphore
            let deferred_submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[gbuffer_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[demo.draw_command_buffers[present_idx as usize]])
                .signal_semaphores(&[demo.rendering_complete_semaphore])
                .build();

            demo.device.queue_submit(demo.present_queue, &[deferred_submit_info], vk::Fence::null())
                .expect("Failed to submit queue!");

            let wait_semaphores = [demo.rendering_complete_semaphore];
            let swapchains = [demo.swapchain];
            let image_indices = [present_idx];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
            demo.swapchain_loader
                .queue_present(demo.present_queue, &present_info)
                .unwrap();
        });

        demo.device.unmap_memory(ub_view_data.memory);
        demo.device.unmap_memory(ub_instance_data.memory);

        demo.device.device_wait_idle().unwrap();

        world
            .material_storage
            .iter()
            .filter(|entry| entry.storage_type & material_filter == material_filter)
            .for_each(|entry| {
                demo.device.destroy_pipeline(entry.component.pso, None);
            });

        demo.device.destroy_pipeline_layout(gbuffer_pipeline_layout, None);
        demo.device.destroy_pipeline_layout(deferred_pipeline_layout, None);

        demo.device.destroy_descriptor_set_layout(gbuffer_descriptor_set_layout, None);
        demo.device.destroy_descriptor_set_layout(deferred_descriptor_set_layout, None);

        let mesh_filter = components::ComponentType::MeshComponent as u32;
        world
            .mesh_storage
            .iter()
            .filter(|entry| entry.storage_type & mesh_filter == mesh_filter)
            .for_each(|entry| {
                entry.component.index_buffer.destroy(&demo.device);
                entry.component.vertex_buffer.destroy(&demo.device);
            });

        ub_instance_data.destroy(&demo.device);
        ub_view_data.destroy(&demo.device);
        for framebuffer in framebuffers {
            demo.device.destroy_framebuffer(framebuffer, None);
        }
        demo.device.destroy_render_pass(renderpass, None);

        gbuffer.destroy(&demo.device);
        demo.device.destroy_sampler(color_sampler, None);
        demo.device.destroy_sampler(depth_sampler, None);
    }
}
