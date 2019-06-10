use vulkano as vk;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2]
}
vulkano::impl_vertex!(Vertex, position);

pub struct FullscreenPass {
    gfx_queue: Arc<vk::device::Queue>,
    vertex_buffer: Arc<vk::buffer::CpuAccessibleBuffer<[Vertex]>>,
    pipeline: Arc<vk::pipeline::GraphicsPipelineAbstract + Send + Sync>
}

impl FullscreenPass {
    pub fn new<R>(gfx_queue: Arc<vk::device::Queue>, subpass: vk::framebuffer::Subpass<R>) -> FullscreenPass where R: vulkano::framebuffer::RenderPassAbstract + Send + Sync + 'static {
        let vertex_buffer = vulkano::buffer::CpuAccessibleBuffer::from_iter(
            gfx_queue.device().clone(), 
            vulkano::buffer::BufferUsage::all(),
            [
                Vertex { 
                    position: [-1.0, -1.0]
                },
                Vertex { 
                    position: [-1.0, 3.0]
                },
                Vertex {
                    position: [3.0, -1.0] 
                }
            ].iter().cloned()
        ).expect("Failed to create vertex buffer for fullscreen pass!");

        FullscreenPass {
            gfx_queue: gfx_queue,
            vertex_buffer: vertex_buffer
        }

        /*let pipeline = {
            let vs = 
        }*/
    }
}