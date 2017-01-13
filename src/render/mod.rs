use ::std::sync::Arc;
use ::std::time::Duration;
use ::vulkano;
use ::vulkano::buffer::CpuAccessibleBuffer;
use ::vulkano::command_buffer::pool::{AllocatedCommandBuffer, CommandPool, CommandPoolFinished, StandardCommandPool};
use ::vulkano_win::VkSurfaceBuild;

// Create vertex and fragment shader
mod vs { include!{concat!(env!("OUT_DIR"), "/shaders/src/render/vs.glsl")} }
mod fs { include!{concat!(env!("OUT_DIR"), "/shaders/src/render/fs.glsl")} }
mod renderpass {
    single_pass_renderpass!{
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: ::vulkano::format::Format,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: ::vulkano::format::D16Unorm,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    }
}
mod pipeline_layout {
    pipeline_layout!{
        set0: {
            uniforms: UniformBuffer<::render::vs::ty::Data>
        }
    }
}


pub trait Renderer {
    fn render(&mut self);
}

pub struct Vulkan {
    command_buffers: Vec<Arc<::vulkano::command_buffer::PrimaryCommandBuffer<Arc<StandardCommandPool>>>>,
    pub device: Arc<::vulkano::device::Device>,
    frame_buffers: Vec<Arc<::vulkano::framebuffer::Framebuffer<renderpass::CustomRenderPass>>>,
    pipeline: Arc<::vulkano::pipeline::GraphicsPipeline<::vulkano::pipeline::vertex::TwoBuffersDefinition<::core::Vertex, ::core::Normal>, pipeline_layout::CustomPipeline, renderpass::CustomRenderPass>>,
    pub queue: Arc<::vulkano::device::Queue>,
    renderpass: Arc<renderpass::CustomRenderPass>,
    set: Arc<pipeline_layout::set0::Set>,
    submissions: Vec<Arc<::vulkano::command_buffer::Submission>>,
    swapchain: Arc<::vulkano::swapchain::Swapchain>,
    uniform_buffer: Arc<CpuAccessibleBuffer<vs::ty::Data>>,
    window: ::vulkano_win::Window,
}

impl Vulkan {
    pub fn new() -> Vulkan {
        // Init vulkan instance
        let instance = {
            let extensions = ::vulkano_win::required_extensions();
            vulkano::instance::Instance::new(None, &extensions, None).expect("failed to create instance")
        };

        // Choose first device
        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
                                .next().expect("no device available");
        debug!("Using device: {} (type: {:?})", physical.name(), physical.ty());

        // Create window
        let window = ::winit::WindowBuilder::new().build_vk_surface(&instance).unwrap();

        // Choose GPU queue for draw command execution
        let queue = physical.queue_families().find(|q| q.supports_graphics() &&
                                                       window.surface().is_supported(q).unwrap_or(false))
                                             .expect("couldn't find a graphical queue family");

        // Initialize device
        let (device, mut queues) = {
            let device_ext = vulkano::device::DeviceExtensions {
                khr_swapchain: true,
                .. vulkano::device::DeviceExtensions::none()
            };

            vulkano::device::Device::new(&physical, physical.supported_features(),
                                         &device_ext, [(queue, 0.5)].iter().cloned())
                                    .expect("failed to create device")
        };
        let queue = queues.next().unwrap();

        // Create a swapchain which allocates color buffers for the screen
        let (swapchain, images) = {
            let caps = window.surface().get_capabilities(&physical).expect("failed to get surface capabilities");
            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let present = caps.present_modes.iter().next().unwrap();
            let usage = caps.supported_usage_flags;
            //let alpha = caps.supported_composite_alpha.iter().next().unwrap(); // image alpha/window transparency
            let format = caps.supported_formats[0].0;

            vulkano::swapchain::Swapchain::new(&device, &window.surface(), 3, format, dimensions, 1,
                                               &usage, &queue, vulkano::swapchain::SurfaceTransform::Identity,
                                               vulkano::swapchain::CompositeAlpha::Opaque,
                                               present, true, None).expect("failed to create swapchain")
        };


        let depth_buffer = vulkano::image::attachment::AttachmentImage::transient(&device, images[0].dimensions(),
                                                                                  vulkano::format::D16Unorm).unwrap();

        // Reverse Y Axis
        // perspective(FOV, aspect ratio, near clipping plane, far clipping plane
        let proj = ::cgmath::perspective(::cgmath::Rad(::std::f32::consts::FRAC_PI_2),
                                       { let d = images[0].dimensions(); d[0] as f32 / d[1] as f32 }, 0.01, 100.0);
        let view = ::cgmath::Matrix4::look_at(::cgmath::Point3::new(0.3, 0.3, 1.0),
                                            ::cgmath::Point3::new(0.0, 0.0, 0.0),
                                            ::cgmath::Vector3::new(0.0, 1.0, 0.0));
        let scale = ::cgmath::Matrix4::from_scale(0.5);

        let uniform_buffer = CpuAccessibleBuffer::<vs::ty::Data>::from_data(
                                 &device, &vulkano::buffer::BufferUsage::all(), Some(queue.family()),
                                 vs::ty::Data {
                                     world : <::cgmath::Matrix4<f32> as ::cgmath::SquareMatrix>::identity().into(),
                                     view : (view * scale).into(),
                                     proj : proj.into(),
                                 }).expect("failed to create buffer");

        let vs = vs::Shader::load(&device).expect("failed to create shader module");
        let fs = fs::Shader::load(&device).expect("failed to create shader module");


        let renderpass = renderpass::CustomRenderPass::new(&device, &renderpass::Formats {
            color: (images[0].format(), 1),
            depth: (vulkano::format::D16Unorm, 1)
        }).unwrap();

        let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);

        let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();
        let set = pipeline_layout::set0::Set::new(&descriptor_pool, &pipeline_layout, &pipeline_layout::set0::Descriptors {
            uniforms: &uniform_buffer
        });

        let pipeline = vulkano::pipeline::GraphicsPipeline::new(&device, vulkano::pipeline::GraphicsPipelineParams {
            vertex_input: vulkano::pipeline::vertex::TwoBuffersDefinition::new(),
            vertex_shader: vs.main_entry_point(),
            input_assembly: vulkano::pipeline::input_assembly::InputAssembly::triangle_list(),

            tessellation: None,
            geometry_shader: None,

            viewport: vulkano::pipeline::viewport::ViewportsState::Fixed {
                data: vec![(
                    vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        depth_range: 0.0 .. 1.0,
                        dimensions: [images[0].dimensions()[0] as f32, images[0].dimensions()[1] as f32],
                    },
                    vulkano::pipeline::viewport::Scissor::irrelevant()
                )],
            },

            raster: Default::default(),
            multisample: vulkano::pipeline::multisample::Multisample::disabled(),
            fragment_shader: fs.main_entry_point(),
            depth_stencil: vulkano::pipeline::depth_stencil::DepthStencil::simple_depth_test(),
            blend: vulkano::pipeline::blend::Blend::pass_through(),
            layout: &pipeline_layout,
            render_pass: vulkano::framebuffer::Subpass::from(&renderpass, 0).unwrap(),
        }).unwrap();

        let frame_buffers = images.iter().map(|image| {
            let dimensions = [image.dimensions()[0], image.dimensions()[1], 1];

            // The `AList` struct was generated by the render pass macro above, and contains one
            // member for each attachment.
            let attachments = renderpass::AList {
                color: &image,
                depth: &depth_buffer,
            };

            vulkano::framebuffer::Framebuffer::new(&renderpass, dimensions, attachments).unwrap()
        }).collect::<Vec<_>>();

        Vulkan {
            command_buffers: Vec::with_capacity(0),
            device: device,
            frame_buffers: frame_buffers,
            pipeline: pipeline,
            queue: queue,
            renderpass: renderpass,
            set: set,
            submissions: Vec::new(),
            swapchain: swapchain,
            uniform_buffer: uniform_buffer,
            window: window,
        }
    }

    pub fn set_draw_buffers(&mut self, vertex_buffer: Arc<CpuAccessibleBuffer<[::core::Vertex]>>,
                                       normals_buffer: Arc<CpuAccessibleBuffer<[::core::Normal]>>,
                                       index_buffer: Arc<CpuAccessibleBuffer<[u16]>>) {
        let command_buffers = self.frame_buffers.iter().map(|frame_buffer| {
            vulkano::command_buffer::PrimaryCommandBufferBuilder::new(&self.device, self.queue.family())
                // Enter render pass
                .draw_inline(&self.renderpass, &frame_buffer, renderpass::ClearValues {
                     color: [0.0, 0.0, 1.0, 1.0],
                     depth: 1.0,
                 })
                // Add draw command
                .draw_indexed(&self.pipeline, (&vertex_buffer, &normals_buffer), &index_buffer,
                              &vulkano::command_buffer::DynamicState::none(), &self.set, &())
                // Leave render pass
                .draw_end()
                .build()
        }).collect::<Vec<_>>();

        self.command_buffers = command_buffers;
    }
}

impl Renderer for Vulkan {
    fn render(&mut self) {
        // Clearing the old submissions by keeping alive only the ones whose destructor would block.
        self.submissions.retain(|s| s.destroying_would_block());

        {
            // aquiring write lock for the uniform buffer
            let mut buffer_content = self.uniform_buffer.write(Duration::new(1, 0)).unwrap();

            let rotation = ::cgmath::Matrix3::from_angle_y(::cgmath::Rad(::time::precise_time_ns() as f32 * 0.000000001));

            // since write lock implementd Deref and DerefMut traits,
            // we can update content directly
            buffer_content.world = ::cgmath::Matrix4::from(rotation).into();
        }

        let image_num = self.swapchain.acquire_next_image(Duration::from_millis(1)).unwrap();
        self.submissions.push(vulkano::command_buffer::submit(&self.command_buffers[image_num], &self.queue).unwrap());
        self.swapchain.present(&self.queue, image_num).unwrap();

        for ev in self.window.window().poll_events() {
            match ev {
                ::winit::Event::Closed => return,
                _ => ()
            }
        }
    }
}
