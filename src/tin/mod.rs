#[macro_use]

use ash::extensions::{
    ext::DebugReport,
    khr::{Surface, Swapchain},
};

use ash::extensions::khr::Win32Surface;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device, Entry, Instance};
use std::cell::RefCell;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::ops::Drop;
use std::os::raw::{c_char, c_void};

pub fn record_submit_command_buffer<D: DeviceV1_0, F: FnOnce(&D, vk::CommandBuffer)>(
    device: &D,
    command_buffer: vk::CommandBuffer,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F
) {
    unsafe {
        device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
            .expect("Failed to reset command buffer!");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin command buffer!");
        f(device, command_buffer);
        device.end_command_buffer(command_buffer)
            .expect("Failed to end command buffer!");

        let submit_fence = device.create_fence(&vk::FenceCreateInfo::default(), None)
            .expect("Failed to create fence!");
        let command_buffers = vec![command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        device.queue_submit(submit_queue, &[submit_info.build()], submit_fence)
            .expect("Failed to submit queue!");

        device.wait_for_fences(&[submit_fence], true, std::u64::MAX)
            .expect("Failed to wait for fence!");
        device.destroy_fence(submit_fence, None);
    }
}

unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(entry: &E, instance: &I, window: &winit::Window) -> Result<vk::SurfaceKHR, vk::Result> {
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

fn extension_names() -> Vec<*const i8> {
    vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr(), DebugReport::name().as_ptr()]
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

pub fn find_memorytype_index(memory_req: &vk::MemoryRequirements, memory_prop: &vk::PhysicalDeviceMemoryProperties, flags: vk::MemoryPropertyFlags) -> Option<u32> {
    let best_suitable_index = find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
            property_flags == flags
    });
    if best_suitable_index.is_some() {
        return best_suitable_index;
    }

    find_memorytype_index_f(memory_req, memory_prop, flags, |property_flags, flags| {
        property_flags & flags == flags
    })
}

pub struct DemoBase {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_report_loader: DebugReport,
    pub window: winit::Window,
    pub events_loop: RefCell<winit::EventsLoop>,
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
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,
}

impl DemoBase {
    pub fn render<F: Fn()>(&self, f: F) {
        use winit::*;
        self.events_loop.borrow_mut().run_forever(|event| {
            f();
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            ControlFlow::Break
                        } else {
                            ControlFlow::Continue
                        }
                    }
                    WindowEvent::CloseRequested => winit::ControlFlow::Break,
                    _ => ControlFlow::Continue,
                },
                _ => ControlFlow::Continue,
            }
        });
    }

    pub fn new(width: u32, height: u32) -> Self {
        unsafe {
            let events_loop = winit::EventsLoop::new();
            let window = winit::WindowBuilder::new()
                .with_title("Electrum")
                .with_dimensions(winit::dpi::LogicalSize::new(width as f64, height as f64))
                .build(&events_loop)
                .unwrap();
            let entry = Entry::new().unwrap();
            let app_name = CString::new("Electrum").unwrap();

            let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
            let layers_names_raw: Vec<*const i8> = layer_names.iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let extension_names_raw = extension_names();

            let appinfo = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(0)
                .engine_name(&app_name)
                .engine_version(0)
                .api_version(ash::vk_make_version!(1, 0, 0));

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names_raw);

            let instance: Instance = entry.create_instance(&create_info, None)
                .expect("Failed to create instance!");

            let debug_info = vk::DebugReportCallbackCreateInfoEXT::builder()
                .flags(vk::DebugReportFlagsEXT::ERROR | vk::DebugReportFlagsEXT::WARNING | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING)
                .pfn_callback(Some(vulkan_debug_callback));

            let debug_report_loader = DebugReport::new(&entry, &instance);
            let debug_call_back = debug_report_loader.create_debug_report_callback(&debug_info, None).unwrap();
            let surface = create_surface(&entry, &instance, &window).unwrap();
            let physical_devices = instance.enumerate_physical_devices()
                .expect("Failed to enumerate physical devices!");
            let surface_loader = Surface::new(&entry, &instance);
            let (physical_device, queue_family_index) = physical_devices.iter()
                .map(|pdev| {
                    instance.get_physical_device_queue_family_properties(*pdev)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let pdev_device_surface_support = surface_loader.get_physical_device_surface_support(*pdev, index as u32, surface);
                            let supports_graphics_and_surface = info.queue_flags.contains(vk::QueueFlags::GRAPHICS) && pdev_device_surface_support;
                            match supports_graphics_and_surface {
                                true => Some((*pdev, index)),
                                _ => None
                            }
                        })
                        .nth(0)
                })
                .filter_map(|v| v)
                .nth(0)
                .expect("Failed to find suitable device!");

            let queue_family_index = queue_family_index as u32;
            let device_exension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities)
                .build()];
        }
    }
}