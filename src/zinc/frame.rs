use std::sync::Arc;
use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector3;

use vulkano::framebuffer::RenderPassDesc;

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
    passes: Vec<vulkano::framebuffer::PassDescription>,
    pass_dependencies: Vec<vulkano::framebuffer::PassDependencyDescription>
}

unsafe impl vulkano::framebuffer::RenderPassDesc for FrameGraphRenderPassDesc {
    fn num_attachments(&self) -> usize {
        self.attachments.len()
    }

    fn attachment_desc(&self, id: usize) -> Option<vulkano::framebuffer::AttachmentDescription> {
        if self.attachments.len() > id {
            Some(self.attachments[id].clone())
        } else {
            None
        }
    }

    fn num_subpasses(&self) -> usize {
        self.passes.len()
    }

    fn subpass_desc(&self, id: usize) -> Option<vulkano::framebuffer::PassDescription> {
        if self.passes.len() > id {
            Some(self.passes[id].clone())
        } else {
            None
        }
    }

    fn num_dependencies(&self) -> usize {
        self.num_subpasses().saturating_sub(1)
    }

    fn dependency_desc(&self, id: usize) -> Option<vulkano::framebuffer::PassDependencyDescription> {
        if id + 1 >= self.num_subpasses() {
            return None;
        }

        Some(self.pass_dependencies[id].clone())
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
        attachment_descs: Vec<vulkano::framebuffer::AttachmentDescription>, 
        passes_descs: Vec<vulkano::framebuffer::PassDescription>,
        passes_dependencies_descs: Vec<vulkano::framebuffer::PassDependencyDescription>
    ) -> FrameGraphRenderPassDesc {
        FrameGraphRenderPassDesc {
            attachments: attachment_descs,
            passes: passes_descs,
            pass_dependencies: passes_dependencies_descs
        }
    }
}

impl FrameGraph {
    pub fn new(gfx_queue: Arc<vulkano::device::Queue>, output_format: vulkano::format::Format) -> FrameGraph {
        let render_pass_desc = FrameGraphRenderPassDesc::new(
            vec![
                vulkano::framebuffer::AttachmentDescription {
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
                }
            ],
            vec![
                vulkano::framebuffer::PassDescription {
                    color_attachments: vec![
                        (1, vulkano::image::ImageLayout::ColorAttachmentOptimal), 
                        (2, vulkano::image::ImageLayout::ColorAttachmentOptimal)
                        ],
                    depth_stencil: Some((0, vulkano::image::ImageLayout::DepthStencilAttachmentOptimal)),
                    input_attachments: [].to_vec(),
                    resolve_attachments: [].to_vec(),
                    preserve_attachments: [].to_vec()
                },
                vulkano::framebuffer::PassDescription {
                    color_attachments: vec![(3, vulkano::image::ImageLayout::ColorAttachmentOptimal)],
                    depth_stencil: None,
                    input_attachments: vec![
                        (0, vulkano::image::ImageLayout::ShaderReadOnlyOptimal),
                        (1, vulkano::image::ImageLayout::ShaderReadOnlyOptimal),
                        (2, vulkano::image::ImageLayout::ShaderReadOnlyOptimal)
                        ],
                    resolve_attachments: [].to_vec(),
                    preserve_attachments: [].to_vec()
                }
            ],
            vec![
                vulkano::framebuffer::PassDependencyDescription {
                    source_subpass: 0,
                    destination_subpass: 1,
                    source_stages: vulkano::sync::PipelineStages { all_graphics: true, .. vulkano::sync::PipelineStages::none() },
                    destination_stages: vulkano::sync::PipelineStages { all_graphics: true, .. vulkano::sync::PipelineStages::none() },
                    source_access: vulkano::sync::AccessFlagBits::all(),
                    destination_access: vulkano::sync::AccessFlagBits::all(),
                    by_region: true,
                }
            ]
        );

       let render_pass = Arc::new(render_pass_desc.build_render_pass(gfx_queue.device().clone()).unwrap());

       // --- create attachements, why 1x1 though, TODO investigate
       let attachment_usage = vulkano::image::ImageUsage {
           transient_attachment: true,
           input_attachment: true,
           .. vulkano::image::ImageUsage::none()
       };

       // --- diffuse
       let gbuffer0 = vulkano::image::AttachmentImage::with_usage(
           gfx_queue.device().clone(),
           [1, 1],
           vulkano::format::Format::A2B10G10R10UnormPack32,
           attachment_usage
       ).unwrap();

       // --- normals
       let gbuffer1 = vulkano::image::AttachmentImage::with_usage(
           gfx_queue.device().clone(),
           [1, 1],
           vulkano::format::Format::R16G16B16A16Sfloat,
           attachment_usage
       ).unwrap();

       // --- depth
       let depthbuffer = vulkano::image::AttachmentImage::with_usage(
           gfx_queue.device().clone(),
           [1, 1],
           vulkano::format::Format::D16Unorm,
           attachment_usage
       ).unwrap();

       FrameGraph {
           gfx_queue: gfx_queue,
           render_pass: render_pass,
           depth_buffer: depthbuffer,
           gbuffer0: gbuffer0,
           gbuffer1: gbuffer1,
           output_format: output_format
       }
    }
}