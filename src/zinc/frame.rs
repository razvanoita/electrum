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

pub struct FrameGraphRenderTargetDesc {
    load_op: vulkano::framebuffer::LoadOp,
    store_op: vulkano::framebuffer::StoreOp,
    format: vulkano::format::Format,
    sample_count: u32
}

pub struct FrameGraphRenderPassDesc {
    render_targets: Vec<usize>,
    depth_stencil_target: Option<usize>,
    input_targets: Vec<usize>
}

pub struct FrameGraphDesc {
    render_target_descs: Vec<FrameGraphRenderTargetDesc>,
    render_pass_descs: Vec<FrameGraphRenderPassDesc>
}

unsafe impl vulkano::framebuffer::RenderPassDesc for FrameGraphDesc {
    fn num_attachments(&self) -> usize {
        self.render_target_descs.len()
    }

    fn attachment_desc(&self, id: usize) -> Option<vulkano::framebuffer::AttachmentDescription> {
        if id >= self.render_target_descs.len() {
            return None;
        }

        // --- [todo]Proper setup for layouts
        let layout = match self.render_target_descs[id].format {
            vulkano::format::Format::D16Unorm => vulkano::image::ImageLayout::DepthStencilAttachmentOptimal,
            _ => vulkano::image::ImageLayout::ColorAttachmentOptimal
        };
        Some(
            vulkano::framebuffer::AttachmentDescription {
                format: self.render_target_descs[id].format,
                samples: self.render_target_descs[id].sample_count,
                load: self.render_target_descs[id].load_op,
                store: self.render_target_descs[id].store_op,
                // --- [todo]Proper values for stencil load/store
                stencil_load: self.render_target_descs[id].load_op,
                stencil_store: self.render_target_descs[id].store_op,
                // --- [todo]Proper setup for layouts
                initial_layout: layout,
                final_layout: layout
            }
        )
    }

    fn num_subpasses(&self) -> usize {
        self.render_pass_descs.len()
    }

    fn subpass_desc(&self, id: usize) -> Option<vulkano::framebuffer::PassDescription> {
        if id >= self.render_pass_descs.len() {
            return None;
        }

        Some(
            vulkano::framebuffer::PassDescription {
                color_attachments: self.render_pass_descs[id].render_targets.iter().map(|x| (x.clone(), vulkano::image::ImageLayout::ColorAttachmentOptimal)).collect(),
                depth_stencil: if let Some(x) = self.render_pass_descs[id].depth_stencil_target { Some((x, vulkano::image::ImageLayout::DepthStencilAttachmentOptimal)) } else { None },
                input_attachments: self.render_pass_descs[id].input_targets.iter().map(|x| (x.clone(), vulkano::image::ImageLayout::ShaderReadOnlyOptimal)).collect(),
                // --- [todo]Handle resolve attachments and preserve attachments properly
                resolve_attachments: [].to_vec(),
                preserve_attachments: [].to_vec()
            }
        )
    }

    fn num_dependencies(&self) -> usize {
        self.num_subpasses().saturating_sub(1)
    }

    fn dependency_desc(&self, id: usize) -> Option<vulkano::framebuffer::PassDependencyDescription> {
        if id + 1 >= self.num_subpasses() {
            return None;
        }

        Some(
            vulkano::framebuffer::PassDependencyDescription {
                // --- [todo]Do source and destination properly, this will only work in the beginning for testing purposes
                source_subpass: id,
                destination_subpass: id + 1,
                // --- [todo]Handle everything else properly
                source_stages: vulkano::sync::PipelineStages { all_graphics: true, .. vulkano::sync::PipelineStages::none() },
                destination_stages: vulkano::sync::PipelineStages { all_graphics: true, .. vulkano::sync::PipelineStages::none() },
                source_access: vulkano::sync::AccessFlagBits::all(),
                destination_access: vulkano::sync::AccessFlagBits::all(),
                by_region: true
            }
        )
    }
}

unsafe impl vulkano::framebuffer::RenderPassDescClearValues<Vec<vulkano::format::ClearValue>> for FrameGraphDesc {
    fn convert_clear_values(&self, values: Vec<vulkano::format::ClearValue>) -> Box<Iterator<Item = vulkano::format::ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}

impl FrameGraphDesc {
    pub fn new(rt_descs: Vec<FrameGraphRenderTargetDesc>,  rp_descs: Vec<FrameGraphRenderPassDesc>) -> FrameGraphDesc {
        FrameGraphDesc {
            render_target_descs: rt_descs,
            render_pass_descs: rp_descs,
        }
    }
}

impl FrameGraph {
    pub fn new(gfx_queue: Arc<vulkano::device::Queue>, output_format: vulkano::format::Format) -> FrameGraph {
        let framegraph_desc = FrameGraphDesc::new(
            vec![
                FrameGraphRenderTargetDesc {
                    load_op: vulkano::framebuffer::LoadOp::Clear,
                    store_op: vulkano::framebuffer::StoreOp::DontCare,
                    format: output_format,
                    sample_count: 1
                },
                FrameGraphRenderTargetDesc {
                    load_op: vulkano::framebuffer::LoadOp::Clear,
                    store_op: vulkano::framebuffer::StoreOp::Store,
                    format: vulkano::format::Format::A2B10G10R10UnormPack32,
                    sample_count: 1
                },
                FrameGraphRenderTargetDesc {
                    load_op: vulkano::framebuffer::LoadOp::Clear,
                    store_op: vulkano::framebuffer::StoreOp::Store,
                    format: vulkano::format::Format::R16G16B16A16Sfloat,
                    sample_count: 1
                },
                FrameGraphRenderTargetDesc {
                    load_op: vulkano::framebuffer::LoadOp::Clear,
                    store_op: vulkano::framebuffer::StoreOp::Store,
                    format: vulkano::format::Format::D16Unorm,
                    sample_count: 1
                }
            ],
            vec![
                FrameGraphRenderPassDesc {
                    render_targets: [1, 2].to_vec(),
                    depth_stencil_target: Some(3),
                    input_targets: [].to_vec()
                },
                FrameGraphRenderPassDesc {
                    render_targets: [0].to_vec(),
                    depth_stencil_target: None,
                    input_targets: [1, 2, 3].to_vec()
                }
            ]
        );

       let render_pass = Arc::new(framegraph_desc.build_render_pass(gfx_queue.device().clone()).unwrap());

       // --- [todo]why 1x1 size for targets, why recreate in future?
       let attachment_usage = vulkano::image::ImageUsage {
           transient_attachment: true,
           input_attachment: true,
           .. vulkano::image::ImageUsage::none()
       };

       let albedo = vulkano::image::AttachmentImage::with_usage(
           gfx_queue.device().clone(),
           [1, 1],
           vulkano::format::Format::A2B10G10R10UnormPack32,
           attachment_usage
       ).unwrap();

       let normals = vulkano::image::AttachmentImage::with_usage(
           gfx_queue.device().clone(),
           [1, 1],
           vulkano::format::Format::R16G16B16A16Sfloat,
           attachment_usage
       ).unwrap();

       // --- depth
       let depth = vulkano::image::AttachmentImage::with_usage(
           gfx_queue.device().clone(),
           [1, 1],
           vulkano::format::Format::D16Unorm,
           attachment_usage
       ).unwrap();

       FrameGraph {
           gfx_queue: gfx_queue,
           render_pass: render_pass,
           depth_buffer: depth,
           gbuffer0: albedo,
           gbuffer1: normals,
           output_format: output_format
       }
    }
}