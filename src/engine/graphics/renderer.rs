// Public renderer-owned resource handles.
// NOTE: Handle types live in `graphics/primitives.rs` now.

use crate::engine::graphics::{Material, MaterialHandle, VisualWorld, MeshUploader};
use crate::engine::graphics::mesh::{CpuMesh, CpuVertex};
use crate::engine::graphics::primitives::{BufferHandle, GpuMesh, MeshHandle};
use winit::window::Window;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

use ash::vk;

// Push constants for camera (view/proj) plus a simple global translation in NDC.
// Layout here must match the shader push-constant block.
#[repr(C)]
#[derive(Clone, Copy)]
struct CameraPushConstants {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    // NDC-space translation applied in the vertex shader.
    // We keep this as vec2 and add explicit padding so the overall layout is well-defined.
    global_translation: [f32; 2],
    _pad0: [f32; 2],
}

fn print_mat4(label: &str, m: &[[f32; 4]; 4]) {
    // Keep it very explicit to avoid any row/column-major confusion in debugging.
    println!("{label}:");
    for r in 0..4 {
        println!(
            "  [{:>9.4}, {:>9.4}, {:>9.4}, {:>9.4}]",
            m[r][0], m[r][1], m[r][2], m[r][3]
        );
    }
}

fn bytes_of<T>(v: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((v as *const T) as *const u8, std::mem::size_of::<T>()) }
}

fn debug_handle_u64<T: vk::Handle>(h: T) -> u64 {
    h.as_raw()
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { &*p_callback_data };
    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::Borrowed("<no message>")
    } else {
        unsafe { std::ffi::CStr::from_ptr(callback_data.p_message) }.to_string_lossy()
    };

    eprintln!(
        "[VULKAN][{:?}][{:?}] {}",
        message_severity,
        message_types,
        message
    );

    vk::FALSE
}


pub struct Renderer {
    /// Renderer-owned resource tables. Handles are lightweight indices into these vecs.
    /// (Eventually these become GPU buffers/pipelines and should use a generational handle scheme.)
    buffers: Vec<GpuBuffer>,
    meshes: Vec<GpuMesh>,
    materials: Vec<Material>,

    // --- Instancing (per-instance data buffer) ---
    instance_buffer: Option<GpuBuffer>,
    /// Capacity of `instance_buffer` in number of instances (mat4 per instance).
    instance_buffer_capacity: usize,
    /// Cached packed instance data (column-major model matrix = 16 f32).
    cached_instance_data: Vec<f32>,

    // --- Camera push constants ---
    cached_camera_view: [[f32; 4]; 4],
    cached_camera_proj: [[f32; 4]; 4],

    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    surface: Option<vk::SurfaceKHR>,
    surface_loader: Option<ash::khr::surface::Instance>,
    debug_utils_loader: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
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
            buffers: Vec::new(),
            meshes: Vec::new(),
            materials: vec![
                Material::UNLIT_FULLSCREEN,
                Material::GRADIENT_BG_XY,
                Material::UNLIT_MESH,
            ],

            instance_buffer: None,
            instance_buffer_capacity: 0,
            cached_instance_data: Vec::new(),

            cached_camera_view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            cached_camera_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],

            entry: None,
            instance: None,
            surface: None,
            surface_loader: None,
            debug_utils_loader: None,
            debug_messenger: None,
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

    fn create_host_visible_buffer_raw(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<GpuBuffer, Box<dyn std::error::Error>> {
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

        Ok(GpuBuffer {
            buffer,
            memory,
            size,
        })
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

    fn destroy_gpu_buffer(&self, b: &GpuBuffer) {
        // Safe to call with an uninitialized renderer? We'll only call this after init.
        let Some(device) = self.device.as_ref() else { return; };
        unsafe {
            device.destroy_buffer(b.buffer, None);
            device.free_memory(b.memory, None);
        }
    }

    fn ensure_instance_buffer_capacity(&mut self, instance_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Each instance is a 4x4 f32 matrix.
        let bytes_per_instance = 16usize * std::mem::size_of::<f32>();

        if self.instance_buffer_capacity >= instance_count && self.instance_buffer.is_some() {
            return Ok(());
        }

        // Grow with slack to avoid reallocating every time (simple doubling).
        let mut new_cap = self.instance_buffer_capacity.max(1);
        while new_cap < instance_count {
            new_cap *= 2;
        }

        // Recreate buffer.
        if let Some(old) = self.instance_buffer.take() {
            self.destroy_gpu_buffer(&old);
        }

        let gpu = self.create_host_visible_buffer_raw(
            (new_cap * bytes_per_instance) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;

        self.instance_buffer = Some(gpu);
        self.instance_buffer_capacity = new_cap;
        Ok(())
    }

    fn rebuild_instance_buffer(
        &mut self,
        visual_world: &mut VisualWorld,
        draw_cache_rebuilt: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let needs_rebuild = draw_cache_rebuilt || visual_world.instance_data_dirty();
        if !needs_rebuild {
            return Ok(());
        }

        // Consume dirty flag (even if empty world).
        let _ = visual_world.take_instance_data_dirty();

        let instance_count = visual_world.draw_order().len();
        self.ensure_instance_buffer_capacity(instance_count)?;

        // Pack model matrices in draw_order.
        self.cached_instance_data.clear();
        self.cached_instance_data.reserve(instance_count * 16);

        for &idx in visual_world.draw_order() {
            let (_r, inst) = visual_world.instances()[idx as usize];
            for col in inst.transform.model {
                self.cached_instance_data.extend_from_slice(&col);
            }
        }

        let Some(gpu) = self.instance_buffer else {
            return Ok(());
        };

        // Upload as raw bytes.
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.cached_instance_data.as_ptr() as *const u8,
                self.cached_instance_data.len() * std::mem::size_of::<f32>(),
            )
        };

        let device = self.device.as_ref().ok_or("Device not initialized")?;
        unsafe {
            let ptr = device.map_memory(gpu.memory, 0, gpu.size, vk::MemoryMapFlags::empty())?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
            device.unmap_memory(gpu.memory);
        }
        Ok(())
    }

    /// Internal implementation of mesh upload (shared by public method and trait).
    fn do_upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>> {
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

    /// Upload a CPU mesh into GPU buffers and return a renderer-owned `MeshHandle`.
    ///
    /// Bring-up implementation: uses HOST_VISIBLE|HOST_COHERENT memory directly.
    pub fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>> {
        self.do_upload_mesh(mesh)
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

        // Vertex input:
        // - binding 0: position (vec3) per-vertex
        // - binding 1: model matrix columns (4x vec4) per-instance
        let vertex_bindings: [vk::VertexInputBindingDescription; 2] = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 12,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 64,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ];

        let vertex_attributes: [vk::VertexInputAttributeDescription; 5] = [
            // location 0: in_pos
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            // locations 1..4: mat4 columns
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 16,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 32,
            },
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 1,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 48,
            },
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attributes);
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
                // MeshFactory returns CCW triangles as front faces.
                // Vulkan defines front-face after the viewport transform. With a top-left
                // origin (common in windowing) the Y axis is flipped, which flips winding.
                // So for our current setup, treat CLOCKWISE as front-facing.
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

        // Push constants: view + proj (2x mat4 = 128 bytes).
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<CameraPushConstants>() as u32);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
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
        let draw_cache_rebuilt = visual_world.prepare_draw_cache();

        // Keep GPU instance buffer in sync with VisualWorld draw order + per-instance data.
        self.rebuild_instance_buffer(visual_world, draw_cache_rebuilt)?;

        // Delegate to the actual Vulkan work.
        let r = self.draw_frame(visual_world);

        r
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
        // No fallback debug-draw triangle path.

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

        // Camera push constants: just refresh from the snapshot every frame.
        // This is tiny (128 bytes) and simpler than threading mutability for a dirty flag.
        self.cached_camera_view = visual_world.camera_view();
        self.cached_camera_proj = visual_world.camera_proj();
        // Global translation from the active 2D camera (NDC pan).
        let global_translation = visual_world.camera_translation();

        let cam_pc = CameraPushConstants {
            view: self.cached_camera_view,
            proj: self.cached_camera_proj,
            global_translation,
            _pad0: [0.0, 0.0],
        };

        // Debug: print the exact matrices that will be pushed to the GPU.
        // Enable with: LC_PRINT_CAMERA_MATRICES=1
        // Printed once per run to keep output readable.
        static PRINTED_CAMERA_MATRICES: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if std::env::var("LC_PRINT_CAMERA_MATRICES").ok().as_deref() == Some("1")
            && !PRINTED_CAMERA_MATRICES.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            println!("[Renderer] Camera push constants (view/proj) about to be pushed:");
            print_mat4("view", &cam_pc.view);
            print_mat4("proj", &cam_pc.proj);
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

            let batch_len = visual_world.draw_batches().len();
            for (i, (batch, &pipeline)) in visual_world
                .draw_batches()
                .iter()
                .zip(batch_pipelines.iter())
                .enumerate()
            {
                if batch.count == 0 {
                    continue;
                }

                // One-time debug print to verify that the pipeline layout used for push constants
                // matches the pipeline we bind for this material.
                // Print per-batch info for the first frame only, when enabled.
                if std::env::var("LC_PRINT_PIPELINE_LAYOUTS").ok().as_deref() == Some("1")
                    && self.current_frame == 0
                {
                    let layout_opt = self.material_pipeline_layouts.get(batch.material.0 as usize).and_then(|v| *v);
                    println!(
                        "[Renderer] pipeline/layout debug: material={:?} mesh={:?} pipeline=0x{:x} layout={}",
                        batch.material,
                        batch.mesh,
                        debug_handle_u64(pipeline),
                        layout_opt
                            .map(|l| format!("0x{:x}", debug_handle_u64(l)))
                            .unwrap_or_else(|| "<missing>".to_string()),
                    );
                    println!(
                        "[Renderer] expected push-constant range: stage=VERTEX offset=0 size={} bytes",
                        std::mem::size_of::<CameraPushConstants>()
                    );
                    println!(
                        "[Renderer] batch idx {}/{} start={} count={}",
                        i,
                        batch_len,
                        batch.start,
                        batch.count
                    );
                }

                // Bind pipeline per material.
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);

                // Push camera constants for UNLIT_MESH only. Other pipelines (fullscreen triangle)
                // don't declare this push constant range.
                if batch.material == MaterialHandle::UNLIT_MESH {
                    let layout = self.material_pipeline_layouts[batch.material.0 as usize]
                        .expect("pipeline layout missing for material");
                    device.cmd_push_constants(
                        command_buffer,
                        layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        bytes_of(&cam_pc),
                    );
                }

                // Mesh buffers.
                let Some(mesh) = self.meshes.get(batch.mesh.0 as usize) else {
                    continue;
                };
                let Some(vb) = self.get_buffer(mesh.vertex_buffer) else { continue; };
                let Some(ib) = self.get_buffer(mesh.index_buffer) else { continue; };

                // Instance buffer (mat4 per instance, stored in draw_order order).
                let Some(inst_buf) = self.instance_buffer else {
                    continue;
                };

                // Debug safety: ensure we don't bind an out-of-bounds offset.
                // Each instance is 64 bytes (mat4 of f32).
                debug_assert!(
                    (batch.start as u64) * 64u64 <= inst_buf.size,
                    "instance buffer offset out of bounds: start={} size={}",
                    batch.start,
                    inst_buf.size
                );
                if (batch.start as u64) * 64u64 > inst_buf.size {
                    // Avoid undefined behavior in release too.
                    continue;
                }

                // Bind vertex buffers: binding 0 = positions, binding 1 = instance matrix.
                let vbs = [vb.buffer, inst_buf.buffer];
                let offsets = [0u64, (batch.start as u64) * 64u64];
                device.cmd_bind_vertex_buffers(command_buffer, 0, &vbs, &offsets);

                // Bind index buffer.
                device.cmd_bind_index_buffer(command_buffer, ib.buffer, 0, vk::IndexType::UINT32);

                // Indexed instanced draw.
                device.cmd_draw_indexed(command_buffer, mesh.index_count, batch.count as u32, 0, 0, 0);
                drew_any = true;
            }

            let _ = drew_any;

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

        if std::env::var("LC_LIST_VK_LAYERS").ok().as_deref() == Some("1") {
            let layers = unsafe { entry.enumerate_instance_layer_properties()? };
            println!("[Renderer] Available Vulkan instance layers ({}):", layers.len());
            for lp in layers {
                let name = unsafe { std::ffi::CStr::from_ptr(lp.layer_name.as_ptr()) }
                    .to_string_lossy();
                println!("  - {}", name);
            }
        }
        
        let app_info = vk::ApplicationInfo::default()
            .application_name(c"little-cat")
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(c"little-cat-engine")
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())?.to_vec();

        if std::env::var("LC_LIST_VK_EXTS").ok().as_deref() == Some("1") {
            let exts = unsafe { entry.enumerate_instance_extension_properties(None)? };
            println!("[Renderer] Available Vulkan instance extensions ({}):", exts.len());
            for ep in exts {
                let name = unsafe { std::ffi::CStr::from_ptr(ep.extension_name.as_ptr()) }
                    .to_string_lossy();
                println!("  - {}", name);
            }
        }

        // Validation is easiest to work with when it screams at us in the terminal.
        let enable_validation = std::env::var("LC_VALIDATION").ok().as_deref() == Some("1");

        // Request debug utils so we can hook a messenger (if validation is enabled).
        if enable_validation {
            extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        // Try to enable the standard validation layer if it exists.
        let mut layer_name_ptrs: Vec<*const i8> = Vec::new();
        let validation_layer = std::ffi::CString::new("VK_LAYER_KHRONOS_validation")?;
        if enable_validation {
            let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };
            let has_validation = available_layers.iter().any(|lp| unsafe {
                std::ffi::CStr::from_ptr(lp.layer_name.as_ptr()) == validation_layer.as_c_str()
            });
            if has_validation {
                layer_name_ptrs.push(validation_layer.as_ptr());
                println!("[Renderer] Vulkan validation: ENABLED (VK_LAYER_KHRONOS_validation)");
            } else {
                println!("[Renderer] Vulkan validation requested (LC_VALIDATION=1) but VK_LAYER_KHRONOS_validation not found");
            }
        }
        
        // If validation is on, attach a debug messenger create-info via pNext so it catches
        // messages produced during vkCreateInstance/vkCreateDevice as well.
        let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let mut create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        if !layer_name_ptrs.is_empty() {
            create_info = create_info.enabled_layer_names(&layer_name_ptrs);
        }
        if enable_validation && !layer_name_ptrs.is_empty() {
            create_info = create_info.push_next(&mut debug_create_info);
        }

        let instance = unsafe { entry.create_instance(&create_info, None) }?;

        // Create debug utils messenger (optional).
        if enable_validation && !layer_name_ptrs.is_empty() {
            let debug_utils_loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
            let messenger = unsafe {
                debug_utils_loader.create_debug_utils_messenger(&debug_create_info, None)
            }?;
            self.debug_utils_loader = Some(debug_utils_loader);
            self.debug_messenger = Some(messenger);
        }



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
        let desired_image_count = surface_caps.min_image_count;
        println!("[Renderer] desired_image_count: {}", desired_image_count);
        let image_count = if surface_caps.max_image_count == 0 {
            println!("[Renderer] max_image_count is unlimited");
            desired_image_count
        } else {
            println!("[Renderer]  min_image_count, max_image_count: {}, {}", surface_caps.min_image_count, surface_caps.max_image_count);
            desired_image_count
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
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
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
        // Vulkan expects SPIR-V as a u32 slice. `include_bytes!` gives us bytes that are
        // *not* guaranteed to be suitably aligned for `[u32]`, so we must handle alignment.
        debug_assert!(code.len() % 4 == 0, "SPIR-V bytecode length must be a multiple of 4");

        let word_count = code.len() / 4;
        let aligned = (code.as_ptr() as usize) % std::mem::align_of::<u32>() == 0;

        if aligned {
            // Safe because we checked alignment and length.
            let code_words = unsafe { std::slice::from_raw_parts(code.as_ptr() as *const u32, word_count) };
            let create_info = vk::ShaderModuleCreateInfo::default().code(code_words);
            unsafe { device.create_shader_module(&create_info, None) }
        } else {
            // Fall back to an aligned copy.
            let mut words = Vec::<u32>::with_capacity(word_count);
            for chunk in code.chunks_exact(4) {
                words.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
            let create_info = vk::ShaderModuleCreateInfo::default().code(&words);
            unsafe { device.create_shader_module(&create_info, None) }
        }
    }

    pub fn resize(&mut self, _size: winit::dpi::PhysicalSize<u32>) {
        // TODO: recreate swapchain
    }
}

impl MeshUploader for Renderer {
    fn upload_mesh(&mut self, mesh: &CpuMesh) -> Result<MeshHandle, Box<dyn std::error::Error>> {
        self.do_upload_mesh(mesh)
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

        // Must destroy debug messenger before destroying the instance.
        if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
            unsafe {
                loader.destroy_debug_utils_messenger(messenger, None);
            }
        }

        if let Some(instance) = &self.instance {
            unsafe {
                instance.destroy_instance(None);
            }
        }
    }
}