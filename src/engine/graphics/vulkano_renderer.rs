use crate::engine::graphics::mesh::CpuMesh;
use crate::engine::graphics::primitives::MeshHandle;
use crate::engine::graphics::visual_world::VisualWorld;
use crate::engine::graphics::MeshUploader;
use std::sync::Arc;
use winit::window::Window;

mod vulkano_backend {
    use std::collections::HashMap;
    use std::mem::size_of;
    use std::sync::Arc;

    use crate::engine::graphics::mesh::{CpuMesh, CpuVertex};
    use crate::engine::graphics::pipeline_descriptor_set_layouts::PipelineDescriptorSetLayouts;
    use crate::engine::graphics::primitives::MeshHandle;
    use crate::engine::graphics::visual_world::VisualWorld;
    use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
    use vulkano::command_buffer::{
        allocator::StandardCommandBufferAllocator,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        CopyBufferInfo,
        PrimaryCommandBufferAbstract,
        RenderPassBeginInfo,
        SubpassBeginInfo,
        SubpassEndInfo,
    };
    use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
    use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
    use vulkano::format::ClearValue;
    use vulkano::image::view::ImageView;
    use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
    use vulkano::pipeline::graphics::color_blend::ColorBlendState;
    use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
    use vulkano::pipeline::graphics::multisample::MultisampleState;
    use vulkano::pipeline::graphics::rasterization::RasterizationState;
    use vulkano::pipeline::graphics::subpass::PipelineSubpassType;
    use vulkano::pipeline::graphics::vertex_input::{
        VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
        VertexInputState,
    };
    use vulkano::pipeline::graphics::viewport::{Scissor, Viewport, ViewportState};
    use vulkano::pipeline::layout::{PipelineLayout, PipelineLayoutCreateInfo};
    
    use vulkano::pipeline::{DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineShaderStageCreateInfo};
    use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
    use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
    use vulkano::sync::{self, GpuFuture};
    use vulkano::{Validated, VulkanError};
    use vulkano::DeviceSize;
    use vulkano::format::Format;
    use vulkano_util::context::{VulkanoConfig, VulkanoContext};
    use winit::window::Window;

    mod toon_mesh_vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/engine/graphics/shaders/toon-mesh.vert",
        }
    }

    mod toon_mesh_fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/engine/graphics/shaders/toon-mesh.frag",
        }
    }

    #[derive(BufferContents, Clone, Copy, Debug, Default)]
    #[repr(C, align(16))]
    pub struct CameraUBO {
        pub view: [[f32; 4]; 4],
        pub proj: [[f32; 4]; 4],
        pub global_translation: [f32; 2],
        pub _pad0: [f32; 2],
    }

    #[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex, Clone, Copy, Debug, Default)]
    #[repr(C)]
    pub struct InstanceData {
        #[format(R32G32B32A32_SFLOAT)]
        pub i_model_c0: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        pub i_model_c1: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        pub i_model_c2: [f32; 4],
        #[format(R32G32B32A32_SFLOAT)]
        pub i_model_c3: [f32; 4],
    }

    pub struct VulkanoGpuMesh {
        #[allow(dead_code)]
        pub vertices: Subbuffer<[CpuVertex]>,
        #[allow(dead_code)]
        pub indices: Subbuffer<[u32]>,
        #[allow(dead_code)]
        pub index_count: u32,
    }

    pub struct VulkanoState {
        #[allow(dead_code)]
        pub context: VulkanoContext,
        #[allow(dead_code)]
        pub window: Arc<Window>,
        #[allow(dead_code)]
        pub surface: Arc<Surface>,
        #[allow(dead_code)]
        pub swapchain: Arc<Swapchain>,
        #[allow(dead_code)]
        pub swapchain_views: Vec<Arc<ImageView>>,
        #[allow(dead_code)]
        pub render_pass: Arc<RenderPass>,
        #[allow(dead_code)]
        pub framebuffers: Vec<Arc<Framebuffer>>,

        #[allow(dead_code)]
        pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

        #[allow(dead_code)]
        pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

        #[allow(dead_code)]
        pub set_layouts: PipelineDescriptorSetLayouts,

        #[allow(dead_code)]
        pub meshes: HashMap<MeshHandle, VulkanoGpuMesh>,

        pub pipeline_toon_mesh: Arc<GraphicsPipeline>,

        pub window_resized: bool,
        pub recreate_swapchain: bool,
        pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    }

    impl VulkanoState {
        pub fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
            // Prefer the helper context while we're migrating: it enables surface extensions
            // and sets up graphics/compute queues and allocators.
            let context = VulkanoContext::new(VulkanoConfig::default());
            let device = context.device().clone();

            let surface = Surface::from_window(device.instance().clone(), window.clone())?;

            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())?;
            let image_format = device
                .physical_device()
                .surface_formats(&surface, Default::default())?
                .first()
                .ok_or("no supported surface formats")?
                .0;

            let mut min_image_count = 2u32.max(surface_capabilities.min_image_count);
            if let Some(max_image_count) = surface_capabilities.max_image_count {
                min_image_count = min_image_count.min(max_image_count);
            }

            let (swapchain, images) = Swapchain::new(device.clone(), surface.clone(), {
                let create_info = SwapchainCreateInfo {
                    // Keep swapchain buffering as low as possible (prefer 2) while
                    // respecting surface min/max limits.
                    min_image_count,
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: vulkano::image::ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .ok_or("no supported composite alpha")?,
                    ..Default::default()
                };
                create_info
            })?;

            let swapchain_views = images
                .into_iter()
                .map(|image| ImageView::new_default(image).map_err(|e| e.into()))
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

            let render_pass = vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        format: swapchain.image_format(),
                        samples: 1,
                        load_op: Clear,
                        store_op: Store,
                    },
                },
                pass: {
                    color: [color],
                    depth_stencil: {},
                }
            )?;

            let framebuffers = swapchain_views
                .iter()
                .map(|view| {
                    Framebuffer::new(
                        render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view.clone()],
                            ..Default::default()
                        },
                    )
                    .map_err(|e| e.into())
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

            let set_layouts = PipelineDescriptorSetLayouts::new(device.clone())?;

            let vs = toon_mesh_vs::load(device.clone())?;
            let fs = toon_mesh_fs::load(device.clone())?;

            let stages = vec![
                PipelineShaderStageCreateInfo::new(
                    vs.entry_point("main")
                        .ok_or("missing toon-mesh.vert entry point")?,
                ),
                PipelineShaderStageCreateInfo::new(
                    fs.entry_point("main")
                        .ok_or("missing toon-mesh.frag entry point")?,
                ),
            ];

            let layout = PipelineLayout::new(
                device.clone(),
                PipelineLayoutCreateInfo {
                    set_layouts: vec![set_layouts.global.clone(), set_layouts.material.clone()],
                    ..Default::default()
                },
            )?;

            // Important: `CpuVertex` currently contains more than just position (e.g. UV),
            // but the shader expects ONLY location 0 to be position. We therefore declare
            // only the attributes we actually use, and use locations 1-4 for per-instance data.
            let vertex_input_state = VertexInputState::new()
                .binding(
                    0,
                    VertexInputBindingDescription {
                        stride: size_of::<CpuVertex>() as u32,
                        input_rate: VertexInputRate::Vertex,
                        ..Default::default()
                    },
                )
                .binding(
                    1,
                    VertexInputBindingDescription {
                        stride: size_of::<InstanceData>() as u32,
                        input_rate: VertexInputRate::Instance { divisor: 1 },
                        ..Default::default()
                    },
                )
                .attribute(
                    0,
                    VertexInputAttributeDescription {
                        binding: 0,
                        format: Format::R32G32B32_SFLOAT,
                        offset: 0,
                        ..Default::default()
                    },
                )
                .attribute(
                    1,
                    VertexInputAttributeDescription {
                        binding: 1,
                        format: Format::R32G32B32A32_SFLOAT,
                        offset: 0,
                        ..Default::default()
                    },
                )
                .attribute(
                    2,
                    VertexInputAttributeDescription {
                        binding: 1,
                        format: Format::R32G32B32A32_SFLOAT,
                        offset: 16,
                        ..Default::default()
                    },
                )
                .attribute(
                    3,
                    VertexInputAttributeDescription {
                        binding: 1,
                        format: Format::R32G32B32A32_SFLOAT,
                        offset: 32,
                        ..Default::default()
                    },
                )
                .attribute(
                    4,
                    VertexInputAttributeDescription {
                        binding: 1,
                        format: Format::R32G32B32A32_SFLOAT,
                        offset: 48,
                        ..Default::default()
                    },
                );

            let subpass = Subpass::from(render_pass.clone(), 0).ok_or("missing subpass 0")?;
            let mut pipeline_ci = vulkano::pipeline::graphics::GraphicsPipelineCreateInfo::layout(layout);
            pipeline_ci.stages = stages.into();
            pipeline_ci.vertex_input_state = Some(vertex_input_state);
            pipeline_ci.input_assembly_state = Some(InputAssemblyState::default());
            pipeline_ci.viewport_state = Some(ViewportState::default());
            pipeline_ci.rasterization_state = Some(RasterizationState::default());
            pipeline_ci.multisample_state = Some(MultisampleState::default());
            pipeline_ci.depth_stencil_state = None;
            pipeline_ci.color_blend_state = Some(ColorBlendState::with_attachment_states(1, Default::default()));
            pipeline_ci.dynamic_state = [DynamicState::Viewport, DynamicState::Scissor]
                .into_iter()
                .collect();
            pipeline_ci.subpass = Some(PipelineSubpassType::BeginRenderPass(subpass));

            let pipeline_toon_mesh = GraphicsPipeline::new(device.clone(), None, pipeline_ci)?;

            let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
                device.clone(),
                Default::default(),
            ));

            let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
                device.clone(),
                Default::default(),
            ));

            Ok(Self {
                context,
                window,
                surface,
                swapchain,
                swapchain_views,
                render_pass,
                framebuffers,

                command_buffer_allocator,
                descriptor_set_allocator,
                meshes: HashMap::new(),

                set_layouts,

                pipeline_toon_mesh,

                window_resized: false,
                recreate_swapchain: false,
                previous_frame_end: Some(sync::now(device).boxed()),
            })
        }

        fn recreate_swapchain_if_needed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if !(self.window_resized || self.recreate_swapchain) {
                return Ok(());
            }

            self.recreate_swapchain = false;
            let new_dimensions = self.window.inner_size();
            if new_dimensions.width == 0 || new_dimensions.height == 0 {
                // Avoid recreating with a zero-sized swapchain while minimized.
                return Ok(());
            }

            let (new_swapchain, new_images) 
                = match self.swapchain.recreate(SwapchainCreateInfo 
            {
                image_extent: new_dimensions.into(),
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                Err(e) => {
                    self.recreate_swapchain = true;
                    println!(
                        "[VulkanoRenderer] failed to recreate swapchain: {}",
                        Validated::unwrap(e)
                    );
                    return Ok(());
                }
            };

            self.swapchain = new_swapchain;
            self.swapchain_views = new_images
                .into_iter()
                .map(|image| ImageView::new_default(image)
                .map_err(|e| e.into()))
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

            self.framebuffers = self
                .swapchain_views
                .iter()
                .map(|view| {
                    Framebuffer::new(
                        self.render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view.clone()],
                            ..Default::default()
                        },
                    )
                    .map_err(|e| e.into())
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

            self.window_resized = false;
            Ok(())
        }

        pub fn render_visual_world(&mut self, visual_world: &mut VisualWorld) 
            -> Result<(), Box<dyn std::error::Error>> 
        {
            self.recreate_swapchain_if_needed()?;

            let device = self.context.device().clone();
            let queue = self.context.graphics_queue().clone();

            if let Some(previous_frame_end) = 
                self.previous_frame_end.as_mut() 
            {
                previous_frame_end.cleanup_finished();
            }

            let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(self.swapchain.clone(), None)
                    .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        self.recreate_swapchain = true;
                        return Ok(());
                    }
                    Err(e) => return Err(Box::new(e)),
                };

            if suboptimal {
                self.recreate_swapchain = true;
            }

            // Always rebuild draw cache cheaply.
            visual_world.prepare_draw_cache();

            // Build instance buffer in draw order so each DrawBatch maps to a contiguous range.
            let instance_count = visual_world.draw_order().len();
            let instances_ref = visual_world.instances();
            let instance_data_iter =    visual_world.draw_order()
                                                                                        .iter()
                                                                                        .map(|&idx| {
                let (_, transform) = instances_ref[idx as usize];
                let m = transform.model;
                InstanceData {
                    i_model_c0: m[0],
                    i_model_c1: m[1],
                    i_model_c2: m[2],
                    i_model_c3: m[3],
                }
            });

            let instance_buffer: Subbuffer<[InstanceData]> = Buffer::from_iter(
                self.context.memory_allocator().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                        MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                instance_data_iter,
            )?;

            let framebuffer = self.framebuffers[image_i as usize].clone();
            let mut render_pass_begin = RenderPassBeginInfo::framebuffer(framebuffer);
            render_pass_begin.clear_values = vec![Some(ClearValue::from([0.0f32, 0.0, 0.0, 1.0]))];

            let extent = self.swapchain.image_extent();
            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: [extent[0] as f32, extent[1] as f32],
                depth_range: 0.0..=1.0,
                ..Default::default()
            };

            // Camera uniform buffer (set=0, binding=0).
            let camera_ubo = CameraUBO {
                view: visual_world.camera_view(),
                proj: visual_world.camera_proj(),
                global_translation: visual_world.camera_translation(),
                _pad0: [0.0, 0.0],
            };

            let camera_buffer: Subbuffer<CameraUBO> = Buffer::from_data(
                self.context.memory_allocator().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                        MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                camera_ubo,
            )?;

            // Lights storage buffer (set=0, binding=1). Placeholder for now.
            // Layout is intentionally minimal until LightSystem is wired in.
            let lights_buffer: Subbuffer<[u32; 4]> = Buffer::from_data(
                self.context.memory_allocator().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                        MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                [0, 0, 0, 0],
            )?;

            let global_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.set_layouts.global.clone(),
                [
                    WriteDescriptorSet::buffer(0, camera_buffer),
                    WriteDescriptorSet::buffer(1, lights_buffer),
                ],
                [],
            )?;

            #[derive(BufferContents, Clone, Copy, Debug, Default)]
            #[repr(C, align(16))]
            struct MaterialUBO {
                base_color: [f32; 4],
                light_dir_ws: [f32; 3],
                quant_steps: f32,
                unlit: u32,
                _pad0: [f32; 3],
            }

            let material_ubo = MaterialUBO {
                base_color: [1.0, 0.7, 0.2, 1.0],
                light_dir_ws: [0.6, 0.4, 0.7],
                quant_steps: 4.0,
                unlit: 0,
                _pad0: [0.0, 0.0, 0.0],
            };

            let material_buffer: Subbuffer<MaterialUBO> = Buffer::from_data(
                self.context.memory_allocator().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                        MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                material_ubo,
            )?;

            let material_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.set_layouts.material.clone(),
                [WriteDescriptorSet::buffer(0, material_buffer)],
                [],
            )?;

            let mut cbb = AutoCommandBufferBuilder::primary(
                self.command_buffer_allocator.clone(),
                queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )?;

            cbb.begin_render_pass(render_pass_begin, SubpassBeginInfo::default())?;

            cbb.set_viewport(0, vec![viewport].into())?;
            cbb.set_scissor(
                0,
                vec![Scissor {
                    offset: [0, 0],
                    extent: [extent[0], extent[1]],
                    ..Default::default()
                }]
                .into(),
            )?;

            cbb.bind_pipeline_graphics(self.pipeline_toon_mesh.clone())?;
            cbb.bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline_toon_mesh.layout().clone(),
                0,
                (global_set, material_set),
            )?;

            // For now we only support UNLIT_MESH; VisualWorld batches already group by material.
            for batch in visual_world.draw_batches() {
                let Some(mesh) = self.meshes.get(&batch.mesh) else {
                    continue;
                };
                cbb.bind_vertex_buffers(0, (mesh.vertices.clone(), instance_buffer.clone()))?;
                cbb.bind_index_buffer(mesh.indices.clone())?;

                if instance_count > 0 {
                    unsafe {
                        cbb.draw_indexed(
                            mesh.index_count,
                            batch.count as u32,
                            0,
                            0,
                            batch.start as u32,
                        )?;
                    }
                }
            }

            cbb.end_render_pass(SubpassEndInfo::default())?;

            let cb = cbb.build()?;

            let start_future: Box<dyn GpuFuture> = self
                .previous_frame_end
                .take()
                .unwrap_or_else(|| sync::now(device.clone()).boxed());

            let execution = start_future
                .join(acquire_future)
                .then_execute(queue.clone(), cb)?
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_i),
                )
                .then_signal_fence_and_flush();

            match execution.map_err(Validated::unwrap) {
                Ok(future) => {
                    // Keep the future so resources can be cleaned up incrementally.
                    self.previous_frame_end = Some(future.boxed());
                }
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    self.previous_frame_end = Some(sync::now(device).boxed());
                }
                Err(e) => {
                    println!("[VulkanoRenderer] failed to flush future: {e}");
                    self.previous_frame_end = Some(sync::now(device).boxed());
                }
            }

            Ok(())
        }

        pub fn upload_mesh(
            &mut self,
            handle: MeshHandle,
            mesh: &CpuMesh,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if self.meshes.contains_key(&handle) {
                return Ok(());
            }

            if mesh.vertices.is_empty() {
                return Err("mesh has no vertices".into());
            }
            if mesh.indices_u32.is_empty() {
                return Err("mesh has no indices".into());
            }

            let memory_allocator = self.context.memory_allocator().clone();
            let queue = self.context.graphics_queue().clone();

            // Host-visible staging buffers.
            let vertices_src = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                        MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                mesh.vertices.iter().copied(),
            )?;

            let indices_src = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                        MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                mesh.indices_u32.iter().copied(),
            )?;

            // Device-local destination buffers.
            let vertices_dst = Buffer::new_slice::<CpuVertex>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
                mesh.vertices.len() as DeviceSize,
            )?;

            let indices_dst = Buffer::new_slice::<u32>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
                mesh.indices_u32.len() as DeviceSize,
            )?;

            // Copy staging -> device-local.
            let mut cbb = AutoCommandBufferBuilder::primary(
                self.command_buffer_allocator.clone(),
                queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )?;

            cbb.copy_buffer(CopyBufferInfo::buffers(vertices_src, vertices_dst.clone()))?;
            cbb.copy_buffer(CopyBufferInfo::buffers(indices_src, indices_dst.clone()))?;

            let cb = cbb.build()?;

            cb.execute(queue.clone())?
                .then_signal_fence_and_flush()?
                .wait(None)?;

            self.meshes.insert(
                handle,
                VulkanoGpuMesh {
                    vertices: vertices_dst,
                    indices: indices_dst,
                    index_count: mesh.index_count(),
                },
            );

            Ok(())
        }
    }
}

/// Vulkano-only renderer.
pub struct VulkanoRenderer {
    vulkano: Option<vulkano_backend::VulkanoState>,
    next_mesh_handle: u32,
    did_enable_present_loop_log: bool,
}

impl VulkanoRenderer {
    pub fn new() -> Self {
        Self {
            vulkano: None,
            next_mesh_handle: 0,
            did_enable_present_loop_log: false,
        }
    }

    pub fn init_for_window(&mut self, window: &Arc<Window>) -> Result<(), Box<dyn std::error::Error>> {
        if self.vulkano.is_none() {
            self.vulkano = Some(vulkano_backend::VulkanoState::new(window.clone())?);
            println!("[VulkanoRenderer] Vulkano swapchain/render-pass initialized");
        }

        Ok(())
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let _ = size;
        if let Some(vulkano) = self.vulkano.as_mut() {
            vulkano.window_resized = true;
        }
    }

    pub fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>> {
        let Some(vulkano) = self.vulkano.as_mut() else {
            return Err("VulkanoRenderer not initialized (call init_for_window first)".into());
        };

        let handle = MeshHandle(self.next_mesh_handle);
        self.next_mesh_handle = self.next_mesh_handle.wrapping_add(1);

        vulkano.upload_mesh(handle, mesh)?;
        Ok(handle)
    }

    pub fn render_visual_world(&mut self, visual_world: &mut VisualWorld) -> Result<(), Box<dyn std::error::Error>> {
        let Some(vulkano) = self.vulkano.as_mut() else {
            return Err("VulkanoRenderer not initialized (call init_for_window first)".into());
        };

        if !self.did_enable_present_loop_log {
            self.did_enable_present_loop_log = true;
            println!("[VulkanoRenderer] Present loop enabled");
        }

        vulkano.render_visual_world(visual_world)
    }
}

impl MeshUploader for VulkanoRenderer {
    fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>> {
        self.upload_mesh(mesh)
    }
}
