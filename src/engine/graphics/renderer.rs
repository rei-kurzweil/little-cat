// Public renderer-owned resource handles.
// NOTE: Handle types live in `graphics/primitives.rs` now.

use crate::engine::graphics::{Material, MaterialHandle, VisualWorld};
use crate::engine::graphics::mesh::{CpuMesh, CpuVertex};
use crate::engine::graphics::primitives::{BufferHandle, GpuMesh, MeshHandle};
use winit::window::Window;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

use ash::vk;


pub struct Renderer {
    /// Bring-up / debugging: if true, draw a hardcoded triangle even when the scene is empty.
    pub debug_draw_hardcoded_triangle: bool,

    /// Renderer-owned resource tables. Handles are lightweight indices into these vecs.
    /// (Eventually these become GPU buffers/pipelines and should use a generational handle scheme.)
    buffers: Vec<GpuBuffer>,
    meshes: Vec<GpuMesh>,
    materials: Vec<Material>,

    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    surface: Option<vk::SurfaceKHR>,
    surface_loader: Option<ash::khr::surface::Instance>,
    physical_device: Option<vk::PhysicalDevice>,
    device: Option<ash::Device>,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    swapchain: Option<vk::SwapchainKHR>,
    swapchain_loader: Option<ash::khr::swapchain::Device>,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    render_pass: Option<vk::RenderPass>,
    pipeline_layout: Option<vk::PipelineLayout>,
    graphics_pipeline: Option<vk::Pipeline>,
    // Per-material pipeline cache (indexed by MaterialHandle.0).
    material_pipelines: Vec<Option<vk::Pipeline>>,
    material_pipeline_layouts: Vec<Option<vk::PipelineLayout>>,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: Option<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    max_frames_in_flight: usize,
}

#[derive(Debug, Clone, Copy)]
struct GpuBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
}

impl Renderer {
    fn ensure_pipeline_cache_len(&mut self, n: usize) {
        if self.material_pipelines.len() < n {
            self.material_pipelines.resize_with(n, || None);
        }
        if self.material_pipeline_layouts.len() < n {
            self.material_pipeline_layouts.resize_with(n, || None);
        }
    }

    pub fn new() -> Self {
        Self {
            debug_draw_hardcoded_triangle: true,
            buffers: Vec::new(),
            meshes: Vec::new(),
            materials: vec![
                Material::UNLIT_FULLSCREEN,
                Material::GRADIENT_BG_XY,
                Material::UNLIT_MESH,
            ],
            entry: None,
            instance: None,
            surface: None,
            surface_loader: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            present_queue: None,
            swapchain: None,
            swapchain_loader: None,
            swapchain_images: Vec::new(),
            swapchain_image_views: Vec::new(),
            swapchain_format: vk::Format::UNDEFINED,
            swapchain_extent: vk::Extent2D::default(),
            render_pass: None,
            pipeline_layout: None,
            graphics_pipeline: None,
            material_pipelines: Vec::new(),
            material_pipeline_layouts: Vec::new(),
            framebuffers: Vec::new(),
            command_pool: None,
            command_buffers: Vec::new(),
            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            in_flight_fences: Vec::new(),
            current_frame: 0,
            max_frames_in_flight: 2,
        }
    }

    fn get_buffer(&self, h: BufferHandle) -> Option<&GpuBuffer> {
        self.buffers.get(h.0 as usize)
    }

    fn find_memory_type(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        for i in 0..mem_properties.memory_type_count {
            let supported = (type_filter & (1 << i)) != 0;
            let has_props = mem_properties.memory_types[i as usize]
                .property_flags
                .contains(properties);
            if supported && has_props {
                return Some(i);
            }
        }
        None
    }

    fn create_host_visible_buffer(
        &mut self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<BufferHandle, Box<dyn std::error::Error>> {
        let device = self.device.as_ref().ok_or("Device not initialized")?;
        let instance = self.instance.as_ref().ok_or("Instance not initialized")?;
        let physical_device = self.physical_device.ok_or("Physical device not initialized")?;

        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let mem_type = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .ok_or("No suitable HOST_VISIBLE memory type")?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type);

        let memory = unsafe { device.allocate_memory(&alloc_info, None) }?;
        unsafe { device.bind_buffer_memory(buffer, memory, 0) }?;

        let handle = BufferHandle(self.buffers.len() as u32);
        self.buffers.push(GpuBuffer { buffer, memory, size });
        Ok(handle)
    }

    fn write_buffer(&self, h: BufferHandle, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let device = self.device.as_ref().ok_or("Device not initialized")?;
        let b = self.get_buffer(h).ok_or("Invalid BufferHandle")?;
        if bytes.len() as u64 > b.size {
            return Err("write_buffer: source larger than buffer".into());
        }

        unsafe {
            let ptr = device.map_memory(b.memory, 0, b.size, vk::MemoryMapFlags::empty())?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
            device.unmap_memory(b.memory);
        }
        Ok(())
    }

    /// Upload a CPU mesh into GPU buffers and return a renderer-owned `MeshHandle`.
    ///
    /// Bring-up implementation: uses HOST_VISIBLE|HOST_COHERENT memory directly.
    pub fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>> {
        // Vertex data: we ignore UVs for now (per your request) and pack positions only.
        let mut vertex_bytes: Vec<u8> = Vec::with_capacity(mesh.vertices.len() * 12);
        for CpuVertex { pos, .. } in mesh.vertices.iter() {
            for f in pos {
                vertex_bytes.extend_from_slice(&f.to_ne_bytes());
            }
        }

        let index_bytes: Vec<u8> = mesh
            .indices_u32
            .iter()
            .flat_map(|i| i.to_ne_bytes())
            .collect();

        let vb = self.create_host_visible_buffer(
            vertex_bytes.len() as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;
        self.write_buffer(vb, &vertex_bytes)?;

        let ib = self.create_host_visible_buffer(
            index_bytes.len() as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )?;
        self.write_buffer(ib, &index_bytes)?;

        // Vertex layout placeholder (positions only).
        static POS_ONLY_LAYOUT: crate::engine::graphics::primitives::VertexLayout =
            crate::engine::graphics::primitives::VertexLayout {
                stride: 12,
                attributes: &[],
            };

        let gpu_mesh = GpuMesh {
            vertex_buffer: vb,
            index_buffer: ib,
            index_count: mesh.index_count(),
            vertex_layout: &POS_ONLY_LAYOUT,
        };

        let h = MeshHandle(self.meshes.len() as u32);
        self.meshes.push(gpu_mesh);
        Ok(h)
    }

    pub fn material(&self, h: MaterialHandle) -> Option<&Material> {
        self.materials.get(h.0 as usize)
    }

    pub fn mesh(&self, h: MeshHandle) -> Option<&GpuMesh> {
        self.meshes.get(h.0 as usize)
    }

    /// Ensure pipelines for all materials referenced by `visual_world` batches exist.
    pub fn prepare_pipelines(&mut self, visual_world: &VisualWorld) -> Result<(), Box<dyn std::error::Error>> {
        for batch in visual_world.draw_batches() {
            self.ensure_material_pipeline(batch.material)?;
        }
        Ok(())
    }

    /// Lazily build a Vulkan pipeline for a material handle. Cached by handle index.
    pub fn ensure_material_pipeline(
        &mut self,
        material: MaterialHandle,
    ) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
        let idx = material.0 as usize;
        self.ensure_pipeline_cache_len(idx + 1);
    let device = self.device.as_ref().ok_or("Renderer device not initialized")?;
    let render_pass = self.render_pass.ok_or("Renderer render pass not initialized")?;

        if let Some(p) = self.material_pipelines[idx] {
            return Ok(p);
        }

        // For now, map MaterialHandle -> embedded SPIR-V shader pair.
        // Later this should load/compile from Material::vertex_shader / fragment_shader.
        //
        let (vert_spv, frag_spv): (&[u8], &[u8]) = match material {
            MaterialHandle::UNLIT_FULLSCREEN => (
                include_bytes!("shaders/spv/triangle.vert.spv"),
                include_bytes!("shaders/spv/triangle.frag.spv"),
            ),
            MaterialHandle::GRADIENT_BG_XY => (
                include_bytes!("shaders/spv/triangle.vert.spv"),
                include_bytes!("shaders/spv/gradient.frag.spv"),
            ),
            MaterialHandle::UNLIT_MESH => (
                include_bytes!("shaders/spv/unlit-mesh.vert.spv"),
                include_bytes!("shaders/spv/unlit-mesh.frag.spv"),
            ),
            _ => (
                include_bytes!("shaders/spv/triangle.vert.spv"),
                include_bytes!("shaders/spv/triangle.frag.spv"),
            ),
        };

        let vert_shader_module = self.create_shader_module(device, vert_spv)?;
        let frag_shader_module = self.create_shader_module(device, frag_spv)?;

        let main_name = c"main";
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(main_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(main_name),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let extent = self.swapchain_extent;
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            // NOTE: winit coordinates / Vulkan Y flip etc may change this later.
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?
        }[0];

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        self.material_pipeline_layouts[idx] = Some(pipeline_layout);
        self.material_pipelines[idx] = Some(pipeline);

        Ok(pipeline)
    }

    /// Render a frame given some view of world/scene state.
    pub fn render_visual_world(&mut self, visual_world: &mut VisualWorld,) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure draw_order + batches are current if you’re using VisualWorld as a render snapshot.
        visual_world.prepare_draw_cache();

        // Delegate to the actual Vulkan work.
        self.draw_frame(visual_world)
    }

    pub fn draw_frame(&mut self, visual_world: &VisualWorld) -> Result<(), Box<dyn std::error::Error>> {
        // Pre-compute pipelines for the current frame before we borrow Vulkan objects from `self`.
        // This avoids Rust borrow conflicts (we can't mutably borrow `self` while also holding
        // immutable borrows to `device`/`swapchain_loader`).
        self.prepare_pipelines(visual_world)?;
        let mut batch_pipelines: Vec<vk::Pipeline> = Vec::with_capacity(visual_world.draw_batches().len());
        for b in visual_world.draw_batches() {
            batch_pipelines.push(self.ensure_material_pipeline(b.material)?);
        }
        let fallback_pipeline = if self.debug_draw_hardcoded_triangle {
            Some(self.ensure_material_pipeline(MaterialHandle::UNLIT_FULLSCREEN)?)
        } else {
            None
        };

        // Now pull out the Vulkan handles we need for the rest of the frame.
        let swapchain = self.swapchain.ok_or("Swapchain not initialized")?;
        let device = self.device.as_ref().ok_or("Device not initialized")?;
        let swapchain_loader = self.swapchain_loader.as_ref().ok_or("Swapchain loader not initialized")?;

        // Wait for previous frame
        unsafe {
            device.wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX)?;
        }

        // Acquire image
        let (image_index, _) = unsafe {
            swapchain_loader.acquire_next_image(
                swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )?
        };

        unsafe {
            device.reset_fences(&[self.in_flight_fences[self.current_frame]])?;
        }

        // Record command buffer
        let command_buffer = self.command_buffers[self.current_frame];

        unsafe {
            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;

            let begin_info = vk::CommandBufferBeginInfo::default();
            device.begin_command_buffer(command_buffer, &begin_info)?;

            let clear_color = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            };

            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass.unwrap())
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_extent,
                })
                .clear_values(std::slice::from_ref(&clear_color));

            device.cmd_begin_render_pass(command_buffer, &render_pass_info, vk::SubpassContents::INLINE);

            let mut drew_any = false;

            for (batch, &pipeline) in visual_world.draw_batches().iter().zip(batch_pipelines.iter()) {
                if batch.count == 0 {
                    continue;
                }

                // Bind pipeline per material.
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);

                // TODO: bind vertex/index buffers for batch.mesh once GPU buffers exist.
                // TODO: use push constants or an instance buffer for per-instance transforms.

                // For now: hardcoded triangle shader; instance_count comes from the batch.
                device.cmd_draw(command_buffer, 3, batch.count as u32, 0, 0);
                drew_any = true;
            }

            // Bring-up fallback: draw 1 triangle if the scene is empty.
            if !drew_any && self.debug_draw_hardcoded_triangle {
                if let Some(pipeline) = fallback_pipeline {
                    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
                }
                device.cmd_draw(command_buffer, 3, 1, 0, 0);
            }

            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer)?;
        }

        // Submit
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let command_buffers = [command_buffer];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device.queue_submit(
                self.graphics_queue.unwrap(),
                &[submit_info],
                self.in_flight_fences[self.current_frame],
            )?;
        }

        // Present
        let swapchains = [swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            swapchain_loader.queue_present(self.present_queue.unwrap(), &present_info)?;
        }

        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;

        Ok(())
    }

    pub fn init_for_window(&mut self, window: &Arc<Window>) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Create Vulkan entry and instance
        let entry = unsafe { ash::Entry::load()? };
        
        let app_info = vk::ApplicationInfo::default()
            .application_name(c"little-cat")
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(c"little-cat-engine")
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

    let extension_names =
        ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())?.to_vec();
        
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None) }?;



        // 2. Create surface
        let display_handle = window.display_handle()?.as_raw();
        let window_handle = window.window_handle()?.as_raw();

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
        }?;
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        // 3. Pick physical device
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;
        let physical_device = physical_devices[0]; // Just pick first one for now

        // 4. Find queue families
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        
        let graphics_family = queue_families
            .iter()
            .enumerate()
            .find(|(_, props)| props.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(i, _)| i as u32)
            .expect("No graphics queue family");

        let present_family = (0..queue_families.len() as u32)
            .find(|&i| unsafe {
                surface_loader.get_physical_device_surface_support(physical_device, i, surface).unwrap_or(false)
            })
            .expect("No present queue family");

        // 5. Create logical device
        let queue_priorities = [1.0];
        let queue_create_infos = [
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(graphics_family)
                .queue_priorities(&queue_priorities),
        ];

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
        
        let graphics_queue = unsafe { device.get_device_queue(graphics_family, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family, 0) };

        // 6. Create swapchain
        let surface_caps = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
        }?;
        let surface_formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)
        }?;
        let _present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
        }?;

        let surface_format = surface_formats[0];
        let present_mode = vk::PresentModeKHR::FIFO; // Always available

        // Some platforms report a "special" extent that means "pick based on the window".
        // Also clamp to the allowed min/max extents.
        let extent = if surface_caps.current_extent.width != u32::MAX {
            surface_caps.current_extent
        } else {
            let size: winit::dpi::PhysicalSize<u32> = window.inner_size();
            vk::Extent2D {
                width: size
                    .width
                    .clamp(surface_caps.min_image_extent.width, surface_caps.max_image_extent.width),
                height: size
                    .height
                    .clamp(surface_caps.min_image_extent.height, surface_caps.max_image_extent.height),
            }
        };

        // max_image_count == 0 means "no maximum".
        let desired_image_count = surface_caps.min_image_count.saturating_add(1);
        let image_count = if surface_caps.max_image_count == 0 {
            desired_image_count
        } else {
            desired_image_count.min(surface_caps.max_image_count)
        };

        let composite_alpha = if surface_caps
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::OPAQUE)
        {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        } else if surface_caps
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
        } else if surface_caps
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED
        } else {
            vk::CompositeAlphaFlagsKHR::INHERIT
        };

        let (image_sharing_mode, queue_family_indices_vec) = if graphics_family != present_family {
            (vk::SharingMode::CONCURRENT, vec![graphics_family, present_family])
        } else {
            (vk::SharingMode::EXCLUSIVE, Vec::new())
        };

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(composite_alpha)
            .present_mode(present_mode)
            .clipped(true);

        if !queue_family_indices_vec.is_empty() {
            swapchain_create_info = swapchain_create_info.queue_family_indices(&queue_family_indices_vec);
        }

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }?;
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

        // 7. Create image views
        let swapchain_image_views: Vec<_> = swapchain_images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                
                unsafe { device.create_image_view(&create_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 8. Create render pass
        let color_attachment = vk::AttachmentDescription::default()
            .format(surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }?;

        // 9. Create graphics pipeline (simple white triangle shader)
    let vert_shader_code = include_bytes!("shaders/spv/triangle.vert.spv");
    let frag_shader_code = include_bytes!("shaders/spv/triangle.frag.spv");

        let vert_shader_module = self.create_shader_module(&device, vert_shader_code)?;
        let frag_shader_module = self.create_shader_module(&device, frag_shader_code)?;

        let main_name = c"main";
        
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(main_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(main_name),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?
        }[0];

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        // 10. Create framebuffers
        let framebuffers: Vec<_> = swapchain_image_views
            .iter()
            .map(|&view| {
                let attachments = [view];
                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                
                unsafe { device.create_framebuffer(&framebuffer_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 11. Create command pool and buffers
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_family);
        
        let command_pool = unsafe { device.create_command_pool(&pool_info, None) }?;

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.max_frames_in_flight as u32);

        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info) }?;

        // 12. Create sync objects
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();

        for _ in 0..self.max_frames_in_flight {
            image_available_semaphores.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            render_finished_semaphores.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            in_flight_fences.push(unsafe { device.create_fence(&fence_info, None)? });
        }

        // Store everything
        self.entry = Some(entry);
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.surface_loader = Some(surface_loader);
        self.physical_device = Some(physical_device);
        self.device = Some(device);
        self.graphics_queue = Some(graphics_queue);
        self.present_queue = Some(present_queue);
        self.swapchain = Some(swapchain);
        self.swapchain_loader = Some(swapchain_loader);
        self.swapchain_images = swapchain_images;
        self.swapchain_image_views = swapchain_image_views;
        self.swapchain_format = surface_format.format;
        self.swapchain_extent = extent;
        self.render_pass = Some(render_pass);
        self.pipeline_layout = Some(pipeline_layout);
        self.graphics_pipeline = Some(graphics_pipeline);
        self.framebuffers = framebuffers;
        self.command_pool = Some(command_pool);
        self.command_buffers = command_buffers;
        self.image_available_semaphores = image_available_semaphores;
        self.render_finished_semaphores = render_finished_semaphores;
        self.in_flight_fences = in_flight_fences;

        Ok(())
    }

    fn create_shader_module(&self, device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
        let code = unsafe {
            std::slice::from_raw_parts(
                code.as_ptr() as *const u32,
                code.len() / 4,
            )
        };
        
        let create_info = vk::ShaderModuleCreateInfo::default().code(code);
        
        unsafe { device.create_shader_module(&create_info, None) }
    }

    pub fn resize(&mut self, _size: winit::dpi::PhysicalSize<u32>) {
        // TODO: recreate swapchain
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                device.device_wait_idle().ok();

                for &semaphore in &self.image_available_semaphores {
                    device.destroy_semaphore(semaphore, None);
                }
                for &semaphore in &self.render_finished_semaphores {
                    device.destroy_semaphore(semaphore, None);
                }
                for &fence in &self.in_flight_fences {
                    device.destroy_fence(fence, None);
                }

                if let Some(pool) = self.command_pool {
                    device.destroy_command_pool(pool, None);
                }

                for &framebuffer in &self.framebuffers {
                    device.destroy_framebuffer(framebuffer, None);
                }

                if let Some(pipeline) = self.graphics_pipeline {
                    device.destroy_pipeline(pipeline, None);
                }
                if let Some(layout) = self.pipeline_layout {
                    device.destroy_pipeline_layout(layout, None);
                }
                if let Some(render_pass) = self.render_pass {
                    device.destroy_render_pass(render_pass, None);
                }

                for &view in &self.swapchain_image_views {
                    device.destroy_image_view(view, None);
                }

                if let (Some(swapchain), Some(loader)) = (self.swapchain, &self.swapchain_loader) {
                    loader.destroy_swapchain(swapchain, None);
                }

                device.destroy_device(None);
            }
        }

        if let (Some(surface), Some(loader)) = (self.surface, &self.surface_loader) {
            unsafe {
                loader.destroy_surface(surface, None);
            }
        }

        if let Some(instance) = &self.instance {
            unsafe {
                instance.destroy_instance(None);
            }
        }
    }
}