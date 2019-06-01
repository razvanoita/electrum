use std::sync::Arc;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2]
}
vulkano::impl_vertex!(Vertex, position);

pub struct FullscreenPass {
    gfx_queue: Arc<vulkano::device::Queue>,
    vertex_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vertex]>>
}