use ash::util::*;
use ash::vk;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{Device, Entry, Instance};

use cgmath::*;

mod tin;
mod pewter;
mod bendalloy;
mod aluminium;
mod duralumin;

use pewter::Buffer;
use aluminium::components;

use std::default::Default;
use std::ffi::CString;
use std::mem;
use std::mem::align_of;
use std::ops::Mul;
use std::os::raw::c_void;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
struct ViewData {
    projection: cgmath::Matrix4<f32>,
    view: cgmath::Matrix4<f32>
}

fn update_non_dynamic_uniform_buffer(mapped_memory: *mut c_void, alignment: vk::DeviceSize, size: vk::DeviceSize, aspect_ratio: f32) {
    // --- build data
    let projection_matrix = cgmath::perspective(
        cgmath::Rad::from(cgmath::Deg(60.0)), 
        aspect_ratio, 
        0.1, 
        256.0
    );
    let view_matrix = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, -10.0), cgmath::Point3::new(0.0, 0.0, 5.0), cgmath::Vector3::new(0.0, 1.0, 0.0));
    let view_data = [
        ViewData {
            projection: projection_matrix,
            view: view_matrix
        }
    ];
    
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
    instance_data: Vec<cgmath::Matrix4<f32>>
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

fn main() {
    let mut world = duralumin::World::new();
    
    unsafe {
        let demo_app = tin::DemoApp::new(1920, 1080);
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

        // --- create shaders
        demo.add_shader(String::from("copper/shaders/triangle_vert.spv"));
        demo.add_shader(String::from("copper/shaders/triangle_frag.spv"));
        
        // --- create platonic solids
        let copy_command_buffer_0 = demo.get_and_begin_command_buffer();
        let tetrahedron_mesh = bendalloy::mesh(bendalloy::platonic::tetrahedron(), &demo.device, &demo.device_memory_properties, copy_command_buffer_0, demo.present_queue);
        let tetrahedron = world.create_entity()
            .with_component(components::Component::TransformComponent(components::Transform {
                position: cgmath::Vector3{x:0.0, y:0.0, z:0.0},
                rotation: cgmath::Vector3{x:0.0, y:0.0, z:0.0},
                scale: cgmath::Vector3{x:1.0, y:1.0, z:1.0}
            }))
            .with_component(components::Component::MeshComponent(tetrahedron_mesh))
            .with_component(components::Component::VelocityComponent(components::Velocity {
                translation_speed: 1.0,
                rotation_speed: 1.0
            }))
            .with_component(components::Component::MaterialComponent(components::Material {
                vertex_shader: demo.get_shader_module(String::from("copper/shaders/triangle_vert.spv")),
                fragment_shader: demo.get_shader_module(String::from("copper/shaders/triangle_frag.spv")),
                pso: vk::Pipeline::null()
            }))
            .build();

        let copy_command_buffer_1 = demo.get_and_begin_command_buffer();
        let cube_mesh = bendalloy::mesh(bendalloy::platonic::cube(), &demo.device, &demo.device_memory_properties, copy_command_buffer_1, demo.present_queue);
        let cube = world.create_entity()
            .with_component(components::Component::TransformComponent(components::Transform {
                position: cgmath::Vector3{x:3.0, y:0.0, z:0.0},
                rotation: cgmath::Vector3{x:0.0, y:0.0, z:0.0},
                scale: cgmath::Vector3{x:1.0, y:1.0, z:1.0}
            }))
            .with_component(components::Component::MeshComponent(cube_mesh))
            .with_component(components::Component::VelocityComponent(components::Velocity {
                translation_speed: 1.0,
                rotation_speed: 1.0
            }))
            .with_component(components::Component::MaterialComponent(components::Material {
                vertex_shader: demo.get_shader_module(String::from("copper/shaders/triangle_vert.spv")),
                fragment_shader: demo.get_shader_module(String::from("copper/shaders/triangle_frag.spv")),
                pso: vk::Pipeline::null()
            }))
            .build();

        let copy_command_buffer_2 = demo.get_and_begin_command_buffer();
        let octahedron_mesh = bendalloy::mesh(bendalloy::platonic::octahedron(), &demo.device, &demo.device_memory_properties, copy_command_buffer_2, demo.present_queue);
        let octahedron = world.create_entity()
            .with_component(components::Component::TransformComponent(components::Transform {
                position: cgmath::Vector3{x:-3.0, y:0.0, z:0.0},
                rotation: cgmath::Vector3{x:0.0, y:0.0, z:0.0},
                scale: cgmath::Vector3{x:1.0, y:1.0, z:1.0}
            }))
            .with_component(components::Component::MeshComponent(octahedron_mesh))
            .with_component(components::Component::VelocityComponent(components::Velocity {
                translation_speed: 1.0,
                rotation_speed: 2.0
            }))
            .with_component(components::Component::MaterialComponent(components::Material {
                vertex_shader: demo.get_shader_module(String::from("copper/shaders/triangle_vert.spv")),
                fragment_shader: demo.get_shader_module(String::from("copper/shaders/triangle_frag.spv")),
                pso: vk::Pipeline::null()
            }))
            .build();

        let copy_command_buffer_3 = demo.get_and_begin_command_buffer();
        let dodecahedron_mesh = bendalloy::mesh(bendalloy::platonic::dodecahedron(), &demo.device, &demo.device_memory_properties, copy_command_buffer_3, demo.present_queue);
        let dodecahedron = world.create_entity()
            .with_component(components::Component::TransformComponent(components::Transform {
                position: cgmath::Vector3{x:0.0, y:-3.0, z:0.0},
                rotation: cgmath::Vector3{x:0.0, y:0.0, z:0.0},
                scale: cgmath::Vector3{x:1.0, y:1.0, z:1.0}
            }))
            .with_component(components::Component::MeshComponent(dodecahedron_mesh))
            .with_component(components::Component::VelocityComponent(components::Velocity {
                translation_speed: 1.0,
                rotation_speed: 1.5
            }))
            .with_component(components::Component::MaterialComponent(components::Material {
                vertex_shader: demo.get_shader_module(String::from("copper/shaders/triangle_vert.spv")),
                fragment_shader: demo.get_shader_module(String::from("copper/shaders/triangle_frag.spv")),
                pso: vk::Pipeline::null()
            }))
            .build();

        let copy_command_buffer_5 = demo.get_and_begin_command_buffer();
        let icosahedron_mesh = bendalloy::mesh(bendalloy::platonic::icosahedron(), &demo.device, &demo.device_memory_properties, copy_command_buffer_5, demo.present_queue);
        let icosahedron = world.create_entity()
            .with_component(components::Component::TransformComponent(components::Transform {
                position: cgmath::Vector3{x:0.0, y:3.0, z:0.0},
                rotation: cgmath::Vector3{x:0.0, y:0.0, z:0.0},
                scale: cgmath::Vector3{x:1.0, y:1.0, z:1.0}
            }))
            .with_component(components::Component::MeshComponent(icosahedron_mesh))
            .with_component(components::Component::VelocityComponent(components::Velocity {
                translation_speed: 1.0,
                rotation_speed: 3.5
            }))
            .with_component(components::Component::MaterialComponent(components::Material {
                vertex_shader: demo.get_shader_module(String::from("copper/shaders/triangle_vert.spv")),
                fragment_shader: demo.get_shader_module(String::from("copper/shaders/triangle_frag.spv")),
                pso: vk::Pipeline::null()
            }))
            .build();

        // --- create dynamic uniform buffer
        let min_ub_alignment = demo.instance.get_physical_device_properties(demo.physical_device).limits.min_uniform_buffer_offset_alignment;
        let mut dynamic_alignment = std::mem::size_of::<cgmath::Matrix4<f32>>() as u64;
        if (min_ub_alignment > 0) {
			dynamic_alignment = (dynamic_alignment + min_ub_alignment - 1) & !(min_ub_alignment - 1);
		}
        let ub_instance_data = pewter::UniformBuffer::construct(
            &demo.device, 
            &demo.device_memory_properties,
            5 as u64,
            dynamic_alignment, 
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            true
        );
        demo.device.bind_buffer_memory(ub_instance_data.descriptor.buffer, ub_instance_data.memory, 0)
            .unwrap();

        let ub_instance_data_ptr = demo.device.map_memory(ub_instance_data.memory, 0, ub_instance_data.size, vk::MemoryMapFlags::empty())
            .unwrap();
        
        let mut rng = rand::thread_rng();
        let transform_filter = components::ComponentType::TransformComponent as u32;
        world.transform_storage.iter_mut()
            .filter(|entry| entry.storage_type & transform_filter == transform_filter)
            .for_each(|entry| {
                entry.component.rotation = cgmath::Vector3 {
                    x: rng.gen_range(-1.0, 1.0),
                    y: rng.gen_range(-1.0, 1.0),
                    z: rng.gen_range(-1.0, 1.0)
                } * 2.0 * 3.14;
            });

        // --- create non-dynamic uniform buffer
        let ub_view_data = pewter::UniformBuffer::construct(
            &demo.device, 
            &demo.device_memory_properties,
            1,
            mem::size_of::<ViewData>() as u64, 
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            false
        );
        demo.device.bind_buffer_memory(ub_view_data.descriptor.buffer, ub_view_data.memory, 0)
            .unwrap();
        
        let ub_view_data_ptr = demo.device.map_memory(ub_view_data.memory, 0, ub_view_data.descriptor.range, vk::MemoryMapFlags::empty())
            .unwrap();
        update_non_dynamic_uniform_buffer(
            ub_view_data_ptr, 
            mem::size_of::<ViewData>() as u64, 
            ub_view_data.descriptor.range, 
            demo.surface_resolution.width as f32 / demo.surface_resolution.height as f32
        );

        // --- descriptor set layout
        demo.setup_descriptor_set_layout();

        let vs_module: vk::ShaderModule = *demo.shader_modules.get("copper/shaders/triangle_vert.spv").unwrap();
        let fs_module: vk::ShaderModule = *demo.shader_modules.get("copper/shaders/triangle_frag.spv").unwrap();

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: demo.descriptor_set_layouts.as_ptr(),
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
                stride: mem::size_of::<bendalloy::platonic::Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ];
        let vertex_input_attribute_descs = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0, 
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(bendalloy::platonic::Vertex, position) as u32
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(bendalloy::platonic::Vertex, normal) as u32
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(bendalloy::platonic::Vertex, color) as u32
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
        demo.setup_descriptor_pool();

        // --- setup descriptor set
        demo.setup_descriptor_sets();
        let write_descriptor_sets = [
            vk::WriteDescriptorSet::builder()
                .dst_binding(0)
                .buffer_info(&[ub_view_data.descriptor])
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .dst_set(demo.descriptor_sets[0])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_binding(1)
                .buffer_info(&[ub_instance_data.descriptor])
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .dst_set(demo.descriptor_sets[0])
                .build()
        ];
        demo.device.update_descriptor_sets(&write_descriptor_sets, &[]);

        let dt: f32 = 1.0 / 60.0;
        let mut current_time = std::time::SystemTime::now();
        let mut accumulator: f32 = 0.0;

        demo_app.run(
            || {
                let new_time = std::time::SystemTime::now();
                let mut frame_time = new_time
                    .duration_since(current_time)
                    .unwrap()
                    .as_millis() as f32 / 1000.0;
                current_time = new_time;

                accumulator += frame_time;

                while (accumulator >= dt) {
                    let transform_velocity_filter = (components::ComponentType::TransformComponent as u32) | (components::ComponentType::VelocityComponent as u32);
                    world.velocity_storage.iter()
                        .filter(|entry| entry.storage_type & transform_velocity_filter == transform_velocity_filter)
                        .zip(
                            world.transform_storage.iter_mut()
                                .filter(|entry| entry.storage_type & transform_velocity_filter == transform_velocity_filter)
                        )
                        .for_each(|(velocity, transform)| {
                            transform.component.rotation += cgmath::Vector3 {
                                x: velocity.component.rotation_speed * dt,
                                y: velocity.component.rotation_speed * dt,
                                z: velocity.component.rotation_speed * dt
                            };
                        });

                    let instance_data: Vec<cgmath::Matrix4<f32>> = world.transform_storage.iter()
                        .filter(|entry| entry.storage_type & transform_filter == transform_filter)
                        .map(|entry| cgmath::Matrix4::from_translation(entry.component.position)
                                        .mul(cgmath::Matrix4::from_axis_angle(cgmath::Vector3{x:1.0, y:0.0, z:0.0}, cgmath::Rad(entry.component.rotation.x)))
                                        .mul(cgmath::Matrix4::from_axis_angle(cgmath::Vector3{x:0.0, y:1.0, z:0.0}, cgmath::Rad(entry.component.rotation.y)))
                                        .mul(cgmath::Matrix4::from_axis_angle(cgmath::Vector3{x:0.0, y:0.0, z:1.0}, cgmath::Rad(entry.component.rotation.z)))
                        )
                        .collect();

                    update_dynamic_uniform_buffer(
                        ub_instance_data_ptr, 
                        dynamic_alignment, 
                        ub_instance_data.descriptor.range * instance_data.len() as u64, 
                        ub_instance_data.memory, 
                        &demo.device,
                        instance_data
                    );

                    accumulator -= dt;
                }
                
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

                        let mesh_filter = components::ComponentType::MeshComponent as u32;
                        let mut dynamic_offset = 0;
                        world.mesh_storage.iter()
                            .filter(|entry| entry.storage_type & mesh_filter == mesh_filter)
                            .for_each(|entry| {
                                device.cmd_bind_descriptor_sets(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &demo.descriptor_sets, &[dynamic_offset * dynamic_alignment as u32]);
                                device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[entry.component.vertex_buffer.buffer], &[0]);
                                device.cmd_bind_index_buffer(draw_command_buffer, entry.component.index_buffer.buffer, 0, vk::IndexType::UINT32);
                                device.cmd_draw_indexed(draw_command_buffer, entry.component.index_buffer.count as u32, 1, 0, 0, 1);        
                                dynamic_offset+=1;
                            });

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
            }
        );

        demo.device.unmap_memory(ub_view_data.memory);
        demo.device.unmap_memory(ub_instance_data.memory);

        demo.device.device_wait_idle().unwrap();
        for pipeline in gfx_pipelines {
            demo.device.destroy_pipeline(pipeline, None);
        }
        demo.device.destroy_pipeline_layout(pipeline_layout, None);
        demo.device.destroy_shader_module(vs_module, None);
        demo.device.destroy_shader_module(fs_module, None);
        let mesh_filter = components::ComponentType::MeshComponent as u32;
        world.mesh_storage.iter()
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
    }
}
