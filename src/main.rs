use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use cgmath::*;

mod tin;

use std::default::Default;
use std::ffi::CString;
use std::fs::File;
use std::mem;
use std::mem::align_of;
use std::path::Path;

#[derive(Clone, Debug, Copy)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4]
}

#[derive(Clone, Debug, Copy)]
struct UBO {
    view_projection: cgmath::Matrix4<f32>,
    world: cgmath::Matrix4<f32>
}

fn main() {
    unsafe {
        let demo = tin::DemoBase::new(1920, 1080);

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
                load_op: vk::AttachmentLoadOp::LOAD,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            }
        ];

        let color_attachment_refs = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            }
        ];

        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()
        ];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_atachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let renderpass = demo.device.create_render_pass(&renderpass_create_info, None)
            .unwrap();

        let framebuffers: Vec<vk::Framebuffer> = demo.present_image_views.iter()
            .map(|&view| {
                let fb_attachments = [view, demo.depth_image_view];
                let fb_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&fb_attachments)
                    .width(demo.surface_resolution.width)
                    .height(demo.surface_resolution.height)
                    .layers(1);

                demo.device.create_framebuffer(&fb_create_info, None)
                    .unwrap()
            })
            .collect();
        
        let index_buffer_data = [0u32, 1, 2];
        let index_buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(&index_buffer_data) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let index_buffer = demo.device.create_buffer(&index_buffer_info, None)
            .unwrap();
        let index_buffer_mem_req = demo.device.get_buffer_memory_requirements(index_buffer);
        let index_buffer_mem_idx = tin::find_memorytype_index(
            &index_buffer_mem_req, 
            &demo.device_memory_properties, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        )
            .expect("Failed to find memory type for index buffer!");
        let index_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: index_buffer_mem_req.size,
            memory_type_index: index_buffer_mem_idx,
            ..Default::default()
        };
        let index_buffer_mem = demo.device.allocate_memory(&index_allocate_info, None)
            .unwrap();
        let index_ptr = demo.device.map_memory(index_buffer_mem, 0, index_buffer_mem_req.size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut index_slice = Align::new(index_ptr, align_of::<u32>() as u64, index_buffer_mem_req.size);
        index_slice.copy_from_slice(&index_buffer_data);
        demo.device.unmap_memory(index_buffer_mem);
        demo.device.bind_buffer_memory(index_buffer, index_buffer_mem, 0)
            .unwrap();

        let vertices = [
            Vertex {
                position: [-1.0, 1.0, 0.0, 1.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0, 1.0],
                color: [0.0, 0.0, 1.0, 1.0]
            },
            Vertex {
                position: [0.0, -1.0, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0]
            }
        ];
        let vertex_buffer_info = vk::BufferCreateInfo {
            size: 3 * std::mem::size_of::<Vertex>() as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let vertex_buffer = demo.device.create_buffer(&vertex_buffer_info, None)
            .unwrap();
        let vertex_buffer_mem_req = demo.device.get_buffer_memory_requirements(vertex_buffer);
        let vertex_buffer_mem_idx = tin::find_memorytype_index(
            &vertex_buffer_mem_req, 
            &demo.device_memory_properties, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        )
            .expect("Failed to find suitable memory type for vertex buffer!");
        let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: vertex_buffer_mem_req.size,
            memory_type_index: vertex_buffer_mem_idx,
            ..Default::default()
        };
        let vertex_buffer_mem = demo.device.allocate_memory(&vertex_buffer_allocate_info, None)
            .unwrap();
        let vertex_buffer_ptr = demo.device.map_memory(vertex_buffer_mem, 0, vertex_buffer_mem_req.size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut vertex_buffer_align = Align::new(vertex_buffer_ptr, align_of::<Vertex>() as u64, vertex_buffer_mem_req.size);
        vertex_buffer_align.copy_from_slice(&vertices);
        demo.device.unmap_memory(vertex_buffer_mem);
        demo.device.bind_buffer_memory(vertex_buffer, vertex_buffer_mem, 0)
            .unwrap();

        // --- create uniform buffer
        let projection_matrix = cgmath::perspective(
            cgmath::Rad::from(cgmath::Deg(60.0)), 
            demo.surface_resolution.width as f32 / demo.surface_resolution.height as f32, 
            0.1, 
            256.0
        );
        let view_matrix = cgmath::Matrix4::look_at(cgmath::Point3::new(1.0, 1.0, 1.0), cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, 1.0, 0.0));
        let ubo_data = [
            UBO {
                view_projection: projection_matrix * view_matrix,
                world: cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0))
            }
        ];
        let ubo_create_info = vk::BufferCreateInfo {
            size: std::mem::size_of::<UBO>() as u64,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            ..Default::default()
        };
        let ubo = demo.device.create_buffer(&ubo_create_info, None)
            .unwrap();
        let ubo_mem_req = demo.device.get_buffer_memory_requirements(ubo);
        let ubo_mem_idx = tin::find_memorytype_index(
            &ubo_mem_req, 
            &demo.device_memory_properties, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        )
            .expect("Failed to find suitable memory type for uniform buffer!");
        let ubo_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: ubo_mem_req.size,
            memory_type_index: ubo_mem_idx,
            ..Default::default()
        };
        let ubo_mem = demo.device.allocate_memory(&ubo_allocate_info, None)
            .unwrap();
        let ubo_ptr = demo.device.map_memory(ubo_mem, 0, ubo_mem_req.size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut ubo_align = Align::new(ubo_ptr, align_of::<UBO>() as u64, ubo_mem_req.size);
        ubo_align.copy_from_slice(&ubo_data);
        demo.device.unmap_memory(ubo_mem);
        demo.device.bind_buffer_memory(ubo, ubo_mem, 0)
            .unwrap();
        let ubo_desc_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(ubo)
            .offset(0 as u64)
            .range(std::mem::size_of::<UBO>() as u64)
            .build();

        let decriptor_set_layout_binding = vk::DescriptorSetLayoutBinding {
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            binding: 0,
            ..Default::default()
        };
        let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: 1,
            p_bindings: &decriptor_set_layout_binding,
            ..Default::default()
        };
        let descriptor_set_layout = demo.device.create_descriptor_set_layout(&descriptor_set_layout_info, None)
            .unwrap();

        // --- create shaders
        let mut vertex_spv_file = File::open(Path::new("copper/shaders/triangle_vert.spv"))
            .expect("Could not find vertex .spv file!");
        let mut fragment_spv_file = File::open(Path::new("copper/shaders/triangle_frag.spv"))
            .expect("Could not find fragment .spv file!");

        let vs_src = read_spv(&mut vertex_spv_file)
            .expect("Failed to read vertex shader .spv file!");
        let vs_info = vk::ShaderModuleCreateInfo::builder().code(&vs_src);

        let fs_src = read_spv(&mut fragment_spv_file)
            .expect("Failed to read fragment shader .spv file!");
        let fs_info = vk::ShaderModuleCreateInfo::builder().code(&fs_src);

        let vs_module = demo.device.create_shader_module(&vs_info, None)
            .expect("Failed to create vertex shader module!");
        let fs_module = demo.device.create_shader_module(&fs_info, None)
            .expect("Failed to create fragment shader module!");

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &descriptor_set_layout,
            ..Default::default()
        };
        let pipeline_layout = demo.device.create_pipeline_layout(&pipeline_layout_create_info, None)
            .unwrap();
        
        let shader_entry_name = CString::new("main").unwrap();
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vs_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: fs_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }
        ];

        let vertex_input_binding_descs = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ];
        let vertex_input_attribute_descs = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0, 
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32
            }
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_attribute_description_count: vertex_input_attribute_descs.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descs.as_ptr(),
            vertex_binding_description_count: vertex_input_binding_descs.len() as u32,
            p_vertex_binding_descriptions: vertex_input_binding_descs.as_ptr(),
            ..Default::default()
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewports = [
            vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: demo.surface_resolution.width as f32,
                height: demo.surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0
            }
        ];
        let scissors = [
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: demo.surface_resolution.clone()
            }
        ];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state.clone(),
            back: noop_stencil_state.clone(),
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [
            vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::all(),
            }
        ];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let gfx_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(renderpass);

        let gfx_pipelines = demo.device.create_graphics_pipelines(vk::PipelineCache::null(), &[gfx_pipeline_info.build()], None)
            .expect("Failed to create graphics pipelines!");
        let gfx_pipeline = gfx_pipelines[0];

        // --- setup descriptor pool
        let descriptor_pool_sizes = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .build();
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&[descriptor_pool_sizes])
            .max_sets(1)
            .build();
        let descriptor_pool = demo.device.create_descriptor_pool(&descriptor_pool_create_info, None)
            .unwrap();

        // --- setup descriptor set
        let descriptor_set_alloc_info =  vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[descriptor_set_layout])
            .descriptor_pool(descriptor_pool)
            .build();
        let descriptor_sets =  demo.device.allocate_descriptor_sets(&descriptor_set_alloc_info)
            .unwrap();
        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_binding(0)
            .buffer_info(&[ubo_desc_buffer_info])
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_set(descriptor_sets[0])
            .build();
        demo.device.update_descriptor_sets(&[write_descriptor_set], &[]);

        demo.render(|| {
            let (present_idx, _) = demo.swapchain_loader.acquire_next_image(demo.swapchain, std::u64::MAX, demo.present_complete_semaphore, vk::Fence::null())
                .unwrap();
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }
                }
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(renderpass)
                .framebuffer(framebuffers[present_idx as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0}, extent: demo.surface_resolution.clone() })
                .clear_values(&clear_values);

            tin::record_submit_command_buffer(
                &demo.device,
                demo.draw_command_buffer,
                demo.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[demo.present_complete_semaphore],
                &[demo.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(draw_command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                    device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, gfx_pipeline);
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_descriptor_sets(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, descriptor_sets.as_slice(), &[]);
                    device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[vertex_buffer], &[0]);
                    device.cmd_bind_index_buffer(draw_command_buffer, index_buffer, 0, vk::IndexType::UINT32);
                    device.cmd_draw_indexed(draw_command_buffer, index_buffer_data.len() as u32, 1, 0, 0, 1);
                    device.cmd_end_render_pass(draw_command_buffer);
                }
            );

            let wait_semaphores = [demo.rendering_complete_semaphore];
            let swapchains = [demo.swapchain];
            let image_indices = [present_idx];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
            demo.swapchain_loader.queue_present(demo.present_queue, &present_info)
                .unwrap();
        });

        demo.device.device_wait_idle().unwrap();
        for pipeline in gfx_pipelines {
            demo.device.destroy_pipeline(pipeline, None);
        }
        demo.device.destroy_pipeline_layout(pipeline_layout, None);
        demo.device.destroy_shader_module(vs_module, None);
        demo.device.destroy_shader_module(fs_module, None);
        demo.device.free_memory(index_buffer_mem, None);
        demo.device.destroy_buffer(index_buffer, None);
        demo.device.free_memory(vertex_buffer_mem, None);
        demo.device.destroy_buffer(vertex_buffer, None);
        for framebuffer in framebuffers {
            demo.device.destroy_framebuffer(framebuffer, None);
        }
        demo.device.destroy_render_pass(renderpass, None);
    }
}
