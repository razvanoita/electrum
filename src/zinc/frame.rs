use std::sync::Arc;
use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector3;

pub struct FrameGraph {
    gfx_queue: Arc<vulkano::device::Queue>,
    render_pass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    depth_buffer: Arc<vulkano::image::AttachmentImage>,
    gbuffer0: Arc<vulkano::image::AttachmentImage>,
    gbuffer1: Arc<vulkano::image::AttachmentImage>,
    output_format: vulkano::format::Format
}

pub struct FrameGraphRenderPassDesc {
    attachments: Vec<vulkano::framebuffer::AttachmentDescription>,
    passes: Vec<vulkano::framebuffer::PassDescription>
}

unsafe impl vulkano::framebuffer::RenderPassDesc for FrameGraphRenderPassDesc {
    fn num_attachments(&self) -> usize {
        self.attachments.len()
    }

    fn attachment_desc(&self, id: usize) -> Option<vulkano::framebuffer::AttachmentDescription> {
        if self.attachments.len() > id {
            Some(self.attachments[id])
        } else {
            None
        }
    }

    fn num_subpasses(&self) -> usize {
        self.passes.len()
    }

    fn subpass_desc(&self, id: usize) -> Option<vulkano::framebuffer::PassDescription> {
        if self.passes.len() > id {
            Some(self.passes[id])
        } else {
            None
        }
    }

    fn num_dependencies(&self) -> usize {
        
    }
}

unsafe impl vulkano::framebuffer::RenderPassDescClearValues<Vec<vulkano::format::ClearValue>> for FrameGraphRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<vulkano::format::ClearValue>) -> Box<Iterator<Item = vulkano::format::ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}

impl FrameGraphRenderPassDesc {
    pub fn new(
        attachmentDescs: Vec<vulkano::framebuffer::AttachmentDescription>, passesDescs: Vec<vulkano::framebuffer::PassDescription>
    ) -> FrameGraphRenderPassDesc {
        FrameGraphRenderPassDesc {
            attachments: attachmentDescs,
            passes: passesDescs
        }
    }
}

impl FrameGraph {
    pub fn new(gfx_queue: Arc<vulkano::device::Queue>, output_format: vulkano::format::Format) -> FrameGraph {
        let renderPassDesc = FrameGraphRenderPassDesc::new(vec![vulkano::framebuffer::AttachmentDescription {
            format: vulkano::format::Format::D16Unorm,
            samples: 1,
            load: vulkano::framebuffer::LoadOp::Clear,
            store: vulkano::framebuffer::StoreOp::DontCare,
            stencil_load: vulkano::framebuffer::LoadOp::Clear,
            stencil_store: vulkano::framebuffer::StoreOp::DontCare,
            initial_layout: vulkano::image::ImageLayout::DepthStencilAttachmentOptimal,
            final_layout: vulkano::image::ImageLayout::DepthStencilAttachmentOptimal,
        },
        vulkano::framebuffer::AttachmentDescription {
            format: vulkano::format::Format::A2B10G10R10UnormPack32,
            samples: 1,
            load: vulkano::framebuffer::LoadOp::Clear,
            store: vulkano::framebuffer::StoreOp::DontCare,
            stencil_load: vulkano::framebuffer::LoadOp::Clear,
            stencil_store: vulkano::framebuffer::StoreOp::DontCare,
            initial_layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
            final_layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
        },
        vulkano::framebuffer::AttachmentDescription {
            format: vulkano::format::Format::R16G16B16A16Sfloat,
            samples: 1,
            load: vulkano::framebuffer::LoadOp::Clear,
            store: vulkano::framebuffer::StoreOp::DontCare,
            stencil_load: vulkano::framebuffer::LoadOp::Clear,
            stencil_store: vulkano::framebuffer::StoreOp::DontCare,
            initial_layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
            final_layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
        },
        vulkano::framebuffer::AttachmentDescription {
            format: output_format,
            samples: 1,
            load: vulkano::framebuffer::LoadOp::Clear,
            store: vulkano::framebuffer::StoreOp::DontCare,
            stencil_load: vulkano::framebuffer::LoadOp::Clear,
            stencil_store: vulkano::framebuffer::StoreOp::DontCare,
            initial_layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
            final_layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
        }],
        vec![vulkano::framebuffer::PassDescription {
            color_attachments: vec![
                (1, vulkano::image::ImageLayout::ColorAttachmentOptimal), 
                (2, vulkano::image::ImageLayout::ColorAttachmentOptimal)
                ],
            depth_stencil: Some((0, vulkano::image::ImageLayout::DepthStencilAttachmentOptimal)),
            input_attachments: [],
            resolve_attachments: [],
            preserve_attachments: []
        },
        vulkano::framebuffer::PassDescription {
            color_attachments: vec![3, vulkano::image::ImageLayout::ColorAttachmentOptimal],
            depth_stencil: None,
            input_attachments: vec![
                (0, vulkano::image::ImageLayout::ShaderReadOnlyOptimal),
                (1, vulkano::image::ImageLayout::ShaderReadOnlyOptimal),
                (2, vulkano::image::ImageLayout::ShaderReadOnlyOptimal)
                ],
            resolve_attachments: [],
            preserve_attachments: []
        }]);
    }
}