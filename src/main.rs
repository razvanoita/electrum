
extern crate cgmath;
#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;
extern crate winit;

mod vulkano_win;
use vulkano_win::*;

mod zinc;

use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;
use winit::{EventsLoop, WindowBuilder, Event, WindowEvent};

fn main() {
    let extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &extensions, None).unwrap();
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let events_loop = EventsLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
    let window = surface.window();

    let queue_fam = physical.queue_families().find(|&q| {
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }).expect("Couldn't find an appropriate queue familiy!");
    
    let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
    let (device, mut queues) = Device::new(physical, physical.supported_features(), &device_ext, [(queue_fam, 0.5)].iter().cloned()).unwrap();
    let queue = queues.next().unwrap();

    let (mut swapchain, mut images) = {
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let initial_dim = if let Some(dimensions) = window.get_inner_size() {
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        } else {
            return;
        };

        Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format, initial_dim, 1, usage, &queue, SurfaceTransform::Identity, alpha,
            PresentMode::Fifo, true, None).unwrap()
    };

}
