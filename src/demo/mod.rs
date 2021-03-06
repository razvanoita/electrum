use ash::extensions::{
    ext::DebugReport,
    khr::{Surface, Swapchain},
};

use crate::geometry;

use ash::extensions::khr::Win32Surface;
use ash::extensions::nv::RayTracing;

use ash::extensions::nv;
use ash::util::*;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, InstanceV1_1};
use ash::{vk, Device, Entry, Instance};
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::mem;
use std::ops::Drop;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::rc::Rc;

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::sync::mpsc::channel;

use crate::render;
use crate::components;

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::zeroed();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

pub fn record_command_buffer<D: DeviceV1_0, F: FnOnce(&D, vk::CommandBuffer)>(
    device: &D,
    command_buffer: vk::CommandBuffer,
    f: F,
) {
    unsafe {
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Failed to reset command buffer!");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin command buffer!");
        f(device, command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end command buffer!");
    }
}

fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
        vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
    ]
}

pub fn record_submit_command_buffer<D: DeviceV1_0, F: FnOnce(&D, vk::CommandBuffer)>(
    device: &D,
    command_buffer: vk::CommandBuffer,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Failed to reset command buffer!");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin command buffer!");
        f(device, command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end command buffer!");

        let submit_fence = device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .expect("Failed to create fence!");
        let command_buffers = vec![command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        device
            .queue_submit(submit_queue, &[submit_info.build()], submit_fence)
            .expect("Failed to submit queue!");

        device
            .wait_for_fences(&[submit_fence], true, std::u64::MAX)
            .expect("Failed to wait for fence!");
        device.destroy_fence(submit_fence, None);
    }
}

unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> std::result::Result<vk::SurfaceKHR, vk::Result> {
    use std::ptr;
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winit::os::windows::WindowExt;

    let hwnd = window.get_hwnd() as HWND;
    let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        hinstance: hinstance,
        hwnd: hwnd as *const c_void,
    };
    let win32_surface_loader = Win32Surface::new(entry, instance);
    win32_surface_loader.create_win32_surface(&win32_create_info, None)
}

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::FALSE
}

pub fn find_memorytype_index_f<F: Fn(vk::MemoryPropertyFlags, vk::MemoryPropertyFlags) -> bool>(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
    f: F,
) -> Option<u32> {
    let mut memory_type_bits = memory_req.memory_type_bits;
    for (index, ref memory_type) in memory_prop.memory_types.iter().enumerate() {
        if memory_type_bits & 1 == 1 {
            if f(memory_type.property_flags, flags) {
                return Some(index as u32);
            }
        }
        memory_type_bits = memory_type_bits >> 1;
    }
    None
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let best_suitable_index =
        find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
            property_flags == flags
        });
    if best_suitable_index.is_some() {
        return best_suitable_index;
    }

    find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
        property_flags & flags == flags
    })
}

pub struct DemoApp {
    pub window: winit::Window,
    pub events_loop: RefCell<winit::EventsLoop>,
    pub app_name: CString,
    pub window_width: u32,
    pub window_height: u32,
}

pub struct DemoContext {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_report_loader: DebugReport,
    pub debug_call_back: vk::DebugReportCallbackEXT,

    pub physical_device: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub draw_command_buffers: Vec<vk::CommandBuffer>,
    pub dcb_wait_fences: Vec<vk::Fence>,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    shader_modules: HashMap<String, vk::ShaderModule>,

    pub watcher: notify::RecommendedWatcher,
    pub watcher_rx: std::sync::mpsc::Receiver<std::path::PathBuf>,
    event_collecter: Vec<std::path::PathBuf>,
    ready_to_process_asset_events: bool,

    pub raytracing: Rc<nv::RayTracing>,
    pub raytracing_properties: vk::PhysicalDeviceRayTracingPropertiesNV,

    pub diagnostics: vk::NvDeviceDiagnosticCheckpointsFn,
}

#[derive(Clone, Debug, Copy)]
pub enum PSOCreateOption {
    // --- flags
    HasVertexAttributes = 0b0000_0000_0000_0001,

    // --- constants
    NoVertexAttributes = 0b0000_0000_0000_0000,
}

impl DemoApp {
    pub fn run<F: FnMut()>(&self, mut f: F) {
        use winit::*;

        loop {
            f();

            let mut done = false;
            self.events_loop.borrow_mut().poll_events(|ev| {
                if let Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } = ev
                {
                    done = true
                }
            });
            if done {
                return;
            }
        }
    }

    pub fn new(width: u32, height: u32) -> Self {
        let events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_title("Electrum")
            .with_dimensions(winit::dpi::LogicalSize::new(width as f64, height as f64))
            .build(&events_loop)
            .unwrap();
        let app_name = CString::new("Electrum").unwrap();

        DemoApp {
            window: window,
            events_loop: RefCell::new(events_loop),
            app_name: app_name,
            window_width: width,
            window_height: height,
        }
    }

    pub fn build_ctx(&self) -> DemoContext {
        unsafe {
            let entry = Entry::new().unwrap();

            let layer_names = [
                CString::new("VK_LAYER_KHRONOS_validation").unwrap(),
            ];

            let layers_names_raw: Vec<*const i8> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let extension_names_raw = extension_names();

            let appinfo = vk::ApplicationInfo::builder()
                .application_name(&self.app_name)
                .application_version(0)
                .engine_name(&self.app_name)
                .engine_version(0)
                .api_version(ash::vk::make_version(1, 0, 0))
                .build();

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names_raw);

            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance!");

            let debug_info = vk::DebugReportCallbackCreateInfoEXT::builder()
                .flags(
                    vk::DebugReportFlagsEXT::ERROR
                        | vk::DebugReportFlagsEXT::WARNING
                        | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
                )
                .pfn_callback(Some(vulkan_debug_callback));

            let debug_report_loader = DebugReport::new(&entry, &instance);
            let debug_call_back = debug_report_loader
                .create_debug_report_callback(&debug_info, None)
                .unwrap();
            let surface = create_surface(&entry, &instance, &self.window).unwrap();
            let physical_devices = instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices!");
            let surface_loader = Surface::new(&entry, &instance);
            let (physical_device, queue_family_index) = physical_devices
                .iter()
                .map(|pdev| {
                    instance
                        .get_physical_device_queue_family_properties(*pdev)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let pdev_device_surface_support = surface_loader
                                .get_physical_device_surface_support(*pdev, index as u32, surface)
                                .unwrap();
                            let supports_graphics_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && info.queue_flags.contains(vk::QueueFlags::COMPUTE)
                                    && pdev_device_surface_support;
                            match supports_graphics_and_surface {
                                true => Some((*pdev, index)),
                                _ => None,
                            }
                        })
                        .nth(0)
                })
                .filter_map(|v| v)
                .nth(0)
                .expect("Failed to find suitable device!");

            let queue_family_index = queue_family_index as u32;

            let device_extension_names_raw: Vec<*const i8> = vec![
                Swapchain::name().as_ptr(),
                RayTracing::name().as_ptr(),
                vk::KhrMaintenance3Fn::name().as_ptr(),
                vk::ExtDescriptorIndexingFn::name().as_ptr(),
                vk::ExtScalarBlockLayoutFn::name().as_ptr(),
                vk::KhrGetMemoryRequirements2Fn::name().as_ptr(),
                vk::NvDeviceDiagnosticCheckpointsFn::name().as_ptr(),
            ];

            let mut descriptor_indexing =
                vk::PhysicalDeviceDescriptorIndexingFeaturesEXT::builder()
                    .descriptor_binding_variable_descriptor_count(true)
                    .runtime_descriptor_array(true)
                    .build();

            let mut scalar_block = vk::PhysicalDeviceScalarBlockLayoutFeaturesEXT::builder()
                .scalar_block_layout(true)
                .build();

            let mut features2 = vk::PhysicalDeviceFeatures2::default();
            instance
                .fp_v1_1()
                .get_physical_device_features2(physical_device, &mut features2);

            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let ext_prop = instance.enumerate_device_extension_properties(physical_device).unwrap();
            for ext in ext_prop {
                let mut x = 0;
                x += 1;
            }

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities)
                .build()];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features2.features)
                .push_next(&mut scalar_block)
                .push_next(&mut descriptor_indexing)
                .build();

            let device: Device = instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap();

            let present_queue = device.get_device_queue(queue_family_index, 0);

            let surface_formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap();

            let surface_format = surface_formats
                .iter()
                .map(|fmt| match fmt.format {
                    vk::Format::UNDEFINED => vk::SurfaceFormatKHR {
                        format: vk::Format::B8G8R8_UNORM,
                        color_space: fmt.color_space,
                    },
                    _ => fmt.clone(),
                })
                .nth(0)
                .expect("Failed to find a suitable surface format!");

            let surface_capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap();

            let mut desired_image_count = surface_capabilities.min_image_count + 1;

            if surface_capabilities.max_image_count > 0
                && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }

            let surface_resolution = match surface_capabilities.current_extent.width {
                std::u32::MAX => vk::Extent2D {
                    width: self.window_width,
                    height: self.window_height,
                },
                _ => surface_capabilities.current_extent,
            };

            let pre_transform = if surface_capabilities
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap();
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::FIFO)
                .unwrap_or(vk::PresentModeKHR::MAILBOX);

            let swapchain_loader = Swapchain::new(&instance, &device);
            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution.clone())
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);
            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);
            let pool = device.create_command_pool(&pool_create_info, None).unwrap();
            let setup_command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let setup_command_buffers = device
                .allocate_command_buffers(&setup_command_buffer_allocate_info)
                .unwrap();

            let setup_command_buffer = setup_command_buffers[0];

            let present_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
            let present_image_views: Vec<vk::ImageView> = present_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect();

            let draw_command_buffers_allocte_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(present_images.len() as u32)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let draw_command_buffers = device
                .allocate_command_buffers(&draw_command_buffers_allocte_info)
                .unwrap();

            let fence_create_info = vk::FenceCreateInfo::builder()
                    .flags(vk::FenceCreateFlags::SIGNALED)
                    .build();
            let mut wait_fences: Vec<vk::Fence> = Vec::default();
            for i in 0..present_images.len() {
                wait_fences.push(device.create_fence(&fence_create_info, None).unwrap());
            }            

            let device_memory_properties =
                instance.get_physical_device_memory_properties(physical_device);

            let depth_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::D16_UNORM)
                .extent(vk::Extent3D {
                    width: surface_resolution.width,
                    height: surface_resolution.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let depth_image = device.create_image(&depth_image_create_info, None).unwrap();
            let depth_image_mem_req = device.get_image_memory_requirements(depth_image);
            let depth_image_mem_idx = find_memorytype_index(
                &depth_image_mem_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Failed to find suitable memory index for depth image!");
            let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(depth_image_mem_req.size)
                .memory_type_index(depth_image_mem_idx);
            let depth_image_memory = device
                .allocate_memory(&depth_image_allocate_info, None)
                .unwrap();
            device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("Failed to bind depth image memory!");

            record_submit_command_buffer(
                &device,
                setup_command_buffer,
                present_queue,
                &[],
                &[],
                &[],
                |device, setup_command_buffer| {
                    let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                        .image(depth_image)
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1)
                                .build(),
                        );
                    device.cmd_pipeline_barrier(
                        setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barriers.build()],
                    );
                },
            );

            let depth_image_view_info = vk::ImageViewCreateInfo::builder()
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .level_count(1)
                        .layer_count(1)
                        .build(),
                )
                .image(depth_image)
                .format(depth_image_create_info.format)
                .view_type(vk::ImageViewType::TYPE_2D);
            let depth_image_view = device
                .create_image_view(&depth_image_view_info, None)
                .unwrap();

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            let (sender, receiver) = channel::<std::path::PathBuf>();
            let watcher = Watcher::new_immediate(move |res: Result<notify::Event>| match res {
                Ok(event) => {
                    sender.send(event.paths.first().unwrap().clone());
                }
                Err(e) => {
                    println!("Watch error: {:?}", e);
                }
            })
            .unwrap();

            let raytracing_properties = nv::RayTracing::get_properties(&instance, physical_device);
            let raytracing = Rc::new(nv::RayTracing::new(&instance, &device));

            let diagnostics = vk::NvDeviceDiagnosticCheckpointsFn::load(|name| unsafe {
                mem::transmute(instance.get_device_proc_addr(device.handle(), name.as_ptr()))
            });

            DemoContext {
                entry: entry,
                instance: instance,
                device: device,
                queue_family_index: queue_family_index,
                physical_device: physical_device,
                device_memory_properties: device_memory_properties,
                surface_loader: surface_loader,
                surface_format: surface_format,
                present_queue: present_queue,
                surface_resolution: surface_resolution,
                swapchain_loader: swapchain_loader,
                swapchain: swapchain,
                present_images: present_images,
                present_image_views: present_image_views,
                pool: pool,
                draw_command_buffers: draw_command_buffers,
                dcb_wait_fences: wait_fences,
                setup_command_buffer: setup_command_buffer,
                depth_image: depth_image,
                depth_image_view: depth_image_view,
                present_complete_semaphore: present_complete_semaphore,
                rendering_complete_semaphore: rendering_complete_semaphore,
                surface: surface,
                debug_call_back: debug_call_back,
                debug_report_loader: debug_report_loader,
                depth_image_memory: depth_image_memory,
                descriptor_pool: vk::DescriptorPool::null(),
                descriptor_sets: Vec::default(),
                shader_modules: HashMap::default(),
                watcher: watcher,
                watcher_rx: receiver,
                event_collecter: Vec::new(),
                ready_to_process_asset_events: false,
                raytracing: raytracing,
                raytracing_properties: raytracing_properties,
                diagnostics: diagnostics,
            }
        }
    }
}

impl DemoContext {
    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        unsafe {
            let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(self.pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffers = self
                .device
                .allocate_command_buffers(&command_buffer_info)
                .unwrap();
            let command_buffer = command_buffers[0];

            command_buffer
        }
    }

    pub fn get_and_begin_command_buffer(&self) -> vk::CommandBuffer {
        unsafe {
            let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(self.pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffers = self
                .device
                .allocate_command_buffers(&command_buffer_info)
                .unwrap();
            let command_buffer = command_buffers[0];
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            self.device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin transient command buffer!");

            command_buffer
        }
    }

    pub fn create_descriptor_set_layout(&self, bindings: Vec<vk::DescriptorSetLayoutBinding>) -> vk::DescriptorSetLayout {
        unsafe {
            let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo {
                binding_count: bindings.len() as u32,
                p_bindings: bindings.as_ptr(),
                ..Default::default()
            };

            self
                .device
                .create_descriptor_set_layout(&descriptor_set_layout_info, None)
                .unwrap()
        }
    }

    pub fn create_pipeline_layout(&self, descriptor_set_layout: vk::DescriptorSetLayout) -> vk::PipelineLayout {
        unsafe {
            let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
                set_layout_count: 1,
                p_set_layouts: [descriptor_set_layout].as_ptr(),
                ..Default::default()
            };

            self
                .device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        }
    }

    pub fn create_descriptor_pool(&mut self, descriptor_pool_sizes: Vec<vk::DescriptorPoolSize>) {
        unsafe {
            let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
                pool_size_count: descriptor_pool_sizes.len() as u32,
                p_pool_sizes: descriptor_pool_sizes.as_ptr(),
                max_sets: 2,
                ..Default::default()
            };
            self.descriptor_pool = self
                .device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .unwrap();
        }
    }

    pub fn add_shader(&mut self, shader_path: &str) {
        unsafe {
            let mut spv_file =
                File::open(Path::new(&shader_path)).expect("Could not find vertex .spv file!");

            let src = read_spv(&mut spv_file).expect("Failed to read shader .spv file!");
            let info = vk::ShaderModuleCreateInfo::builder().code(&src);

            let module = self
                .device
                .create_shader_module(&info, None)
                .expect("Failed to create vertex shader module!");

            self.shader_modules
                .insert(String::from(shader_path), module);
        }
    }

    pub fn get_shader_module(&self, shader_path: &str) -> vk::ShaderModule {
        *self.shader_modules.get(&String::from(shader_path)).unwrap()
    }

    pub fn reload_shader_module(
        &mut self,
        shader_path: &str,
    ) -> (vk::ShaderModule, vk::ShaderModule) {
        let existing_shader_module = self.get_shader_module(shader_path.clone());
        self.add_shader(shader_path.clone());
        (existing_shader_module, self.get_shader_module(shader_path))
    }

    pub fn create_pso(
        &self,
        vs_module: vk::ShaderModule,
        fs_module: vk::ShaderModule,
        renderpass: vk::RenderPass,
        pipeline_layout: vk::PipelineLayout,
        viewports: [vk::Viewport; 1],
        scissors: [vk::Rect2D; 1],
        color_blend_attachment_states: &[vk::PipelineColorBlendAttachmentState],
        create_flags: PSOCreateOption,
    ) -> vk::Pipeline {
        unsafe {
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
                },
            ];

            let vertex_input_binding_descs = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: mem::size_of::<geometry::Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];
            let vertex_input_attribute_descs = [
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(geometry::Vertex, position) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(geometry::Vertex, normal) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(geometry::Vertex, color) as u32,
                },
            ];
            let vertex_input_state_info = if (create_flags as u32) & (PSOCreateOption::HasVertexAttributes as u32) == (PSOCreateOption::HasVertexAttributes as u32) {
                vk::PipelineVertexInputStateCreateInfo {
                    vertex_attribute_description_count: vertex_input_attribute_descs.len() as u32,
                    p_vertex_attribute_descriptions: vertex_input_attribute_descs.as_ptr(),
                    vertex_binding_description_count: vertex_input_binding_descs.len() as u32,
                    p_vertex_binding_descriptions: vertex_input_binding_descs.as_ptr(),
                    ..Default::default()
                }
            } else {
                vk::PipelineVertexInputStateCreateInfo::default()
            };
            let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };
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
            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);
            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);
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

            let gfx_pipelines = self
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[gfx_pipeline_info.build()],
                    None,
                )
                .expect("Failed to create graphics pipelines!");
            let gfx_pipeline = gfx_pipelines[0];
            gfx_pipeline
        }
    }

    pub fn receive_asset_event(&mut self) {
        const NUM_EVENTS_MODIFIED: usize = 2;
        let event = self.watcher_rx.try_recv();
        if event.is_ok() {
            self.event_collecter.push(event.unwrap());
        } else if self.event_collecter.len() == NUM_EVENTS_MODIFIED {
            self.ready_to_process_asset_events = true;
        }
    }

    pub fn process_asset_event(&mut self) -> Option<String> {
        let mut res: Option<String> = None;
        if self.ready_to_process_asset_events {
            let path = self.event_collecter.pop().unwrap();
            self.event_collecter.clear();

            let asset_name = String::from(path.file_name().unwrap().to_str().unwrap());
            let shader_asset_bin_path: String = String::from("copper/shaders/bin/");
            let key: String = shader_asset_bin_path + &asset_name;
            println!("Shader {:?} changed!", key);

            self.ready_to_process_asset_events = false;

            res = Some(key);
        }
        res
    }
}

impl Drop for DemoContext {
    fn drop(&mut self) {
        unsafe {
            self.shader_modules.iter().for_each(|(k, v)| {
                self.device.destroy_shader_module(*v, None);
            });

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.device.device_wait_idle().unwrap();

            self.device
                .destroy_semaphore(self.present_complete_semaphore, None);
            self.device
                .destroy_semaphore(self.rendering_complete_semaphore, None);

            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.device.destroy_command_pool(self.pool, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_report_loader
                .destroy_debug_report_callback(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}

pub fn end_and_submit_command_buffer(
    device: &ash::Device,
    present_queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
) {
    unsafe {
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to end command buffer!");

        let submit_fence = device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .expect("Failed to create fence!");
        let command_buffers = vec![command_buffer];
        let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffers);

        device
            .queue_submit(present_queue, &[submit_info.build()], submit_fence)
            .expect("Failed to submit queue!");

        device
            .wait_for_fences(&[submit_fence], true, std::u64::MAX)
            .expect("Failed to wait for fence!");
        device.destroy_fence(submit_fence, None);
    }
}
