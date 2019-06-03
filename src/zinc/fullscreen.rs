use std::sync::Arc;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2]
}
vulkano::impl_vertex!(Vertex, position);

pub struct FullscreenPass {
    gfx_queue: Arc<vulkano::device::Queue>,
    vertex_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vertex]>>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipelineAbstract + Send + Sync>
}

impl FullscreenPass {
    pub fn new<R>(gfx_queue: Arc<Queue>, subpass: Subpass<R>) -> FullscreenPass where R: vulkano::framebuffer::RenderPassAbstract + Send + Sync + 'static {
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

        /*let pipeline = {
            let vs = 
        }*/
    }
}