#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate time;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate cgmath;
extern crate collada;

use collada::document::ColladaDocument;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano_win::VkSurfaceBuild;

mod logger;
mod vs { include!{concat!(env!("OUT_DIR"), "/shaders/src/vs.glsl")} }
mod fs { include!{concat!(env!("OUT_DIR"), "/shaders/src/fs.glsl")} }

static COLLADA_FILE: &'static str = "/home/yutoo/monkey.dae";

error_chain!{}

#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32, f32)
}

#[derive(Copy, Clone)]
struct Normal {
    normal: (f32, f32, f32)
}

impl_vertex!(Vertex, position);
impl_vertex!(Normal, normal);

/// Returns (vertex, normals, index) buffers from a collada file
fn get_collada_buffers(file: &ColladaDocument, device: &Arc<vulkano::device::Device>,
                       queue: &vulkano::device::Queue)
    -> (Arc<CpuAccessibleBuffer<[Vertex]>>, Arc<CpuAccessibleBuffer<[Normal]>>, Arc<CpuAccessibleBuffer<[u16]>>) {
    let obj_set = file.get_obj_set().expect("ObjectSet in Collada file not found.");

    // Map collada lib vertices to vulkano/our vertices
    // We have to collect these iterators because from_iter requires the ExactSizeIterator trait
    let vertex_buffer = obj_set.objects.iter()
                                       .flat_map(|obj| obj.vertices.iter())
                                       .map(|vert| Vertex { position: (vert.x as f32,
                                                                       vert.y as f32,
                                                                       vert.z as f32) })
                                       .collect::<Vec<Vertex>>();
    let vertex_buffer = CpuAccessibleBuffer::from_iter(device, &BufferUsage::all(), Some(queue.family()),
                                                       vertex_buffer.iter().cloned())
                                            .expect("failed to create vertex buffer");

    let normal_buffer = obj_set.objects.iter()
                                       .flat_map(|obj| obj.normals.iter())
                                       .map(|norm| Normal { normal: (norm.x as f32,
                                                                     norm.y as f32,
                                                                     norm.z as f32) })
                                       .collect::<Vec<Normal>>();
    let normal_buffer = CpuAccessibleBuffer::from_iter(device, &BufferUsage::all(), Some(queue.family()),
                                                       normal_buffer.iter().cloned())
                                            .expect("failed to create normals buffer");

    // No iterators for tuples, so add to a new array.
    let mut index_buffer = Vec::new();
    for obj in obj_set.objects.iter() {
        for geo in obj.geometry.iter() {
            for &shape in geo.shapes.iter() {
                match shape {
                    collada::Shape::Triangle(u0, u1, u2) => {
                        let (vertex_index, _, _) = u0;
                        index_buffer.push(vertex_index as u16);

                        let (vertex_index, _, _) = u1;
                        index_buffer.push(vertex_index as u16);

                        let (vertex_index, _, _) = u2;
                        index_buffer.push(vertex_index as u16);
                    },
                    _ => panic!("Non-triangle shape!"),
                }
            }
        }
    }
    let index_buffer = CpuAccessibleBuffer::from_iter(device, &BufferUsage::all(), Some(queue.family()),
                                                      index_buffer.iter().cloned())
                                           .expect("failed to create index buffer");


    (vertex_buffer, normal_buffer, index_buffer)
}

fn init_vulkan() {
    // Init vulkan instance
    let instance = {
        let extensions = vulkano_win::required_extensions();
        vulkano::instance::Instance::new(None, &extensions, None).expect("failed to create instance")
    };

    // Choose first device
    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
                            .next().expect("no device available");
    debug!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    // Create window
    let window = winit::WindowBuilder::new().build_vk_surface(&instance).unwrap();

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

    let doc = match ColladaDocument::from_path(Path::new(COLLADA_FILE)) {
        Ok(file) => file,
        Err(e) => { println!("{}", e); return },
    };

    let (vertex_buffer, normals_buffer, index_buffer) = get_collada_buffers(&doc, &device, &queue);

    // Reverse Y Axis
    // perspective(FOV, aspect ratio, near clipping plane, far clipping plane
    let proj = cgmath::perspective(cgmath::Rad(std::f32::consts::FRAC_PI_2),
                                   { let d = images[0].dimensions(); d[0] as f32 / d[1] as f32 }, 0.01, 100.0);
    let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.3, 0.3, 1.0),
                                        cgmath::Point3::new(0.0, 0.0, 0.0),
                                        cgmath::Vector3::new(0.0, 1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(0.5);

    let uniform_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::<vs::ty::Data>::from_data(
                             &device, &vulkano::buffer::BufferUsage::all(), Some(queue.family()),
                             vs::ty::Data {
                                 world : <cgmath::Matrix4<f32> as cgmath::SquareMatrix>::identity().into(),
                                 view : (view * scale).into(),
                                 proj : proj.into(),
                             }).expect("failed to create buffer");

    let vs = vs::Shader::load(&device).expect("failed to create shader module");
    let fs = fs::Shader::load(&device).expect("failed to create shader module");

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

    let renderpass = renderpass::CustomRenderPass::new(&device, &renderpass::Formats {
        color: (images[0].format(), 1),
        depth: (vulkano::format::D16Unorm, 1)
    }).unwrap();

    let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);

    mod pipeline_layout {
        pipeline_layout!{
            set0: {
                uniforms: UniformBuffer<::vs::ty::Data>
            }
        }
    }

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

    let framebuffers = images.iter().map(|image| {
        let dimensions = [image.dimensions()[0], image.dimensions()[1], 1];

        // The `AList` struct was generated by the render pass macro above, and contains one
        // member for each attachment.
        let attachments = renderpass::AList {
            color: &image,
            depth: &depth_buffer,
        };

        vulkano::framebuffer::Framebuffer::new(&renderpass, dimensions, attachments).unwrap()
    }).collect::<Vec<_>>();


    let command_buffers = framebuffers.iter().map(|framebuffer| {
        vulkano::command_buffer::PrimaryCommandBufferBuilder::new(&device, queue.family())
            // Enter render pass
            .draw_inline(&renderpass, &framebuffer, renderpass::ClearValues {
                 color: [0.0, 0.0, 1.0, 1.0],
                 depth: 1.0,
             })
            // Add draw command
            .draw_indexed(&pipeline, (&vertex_buffer, &normals_buffer), &index_buffer,
                          &vulkano::command_buffer::DynamicState::none(), &set, &())
            // Leave render pass
            .draw_end()
            .build()
    }).collect::<Vec<_>>();

    let mut submissions: Vec<Arc<vulkano::command_buffer::Submission>> = Vec::new();


    loop {
        // Clearing the old submissions by keeping alive only the ones whose destructor would block.
        submissions.retain(|s| s.destroying_would_block());

        {
            // aquiring write lock for the uniform buffer
            let mut buffer_content = uniform_buffer.write(Duration::new(1, 0)).unwrap();

            let rotation = cgmath::Matrix3::from_angle_y(cgmath::Rad(time::precise_time_ns() as f32 * 0.000000001));

            // since write lock implementd Deref and DerefMut traits,
            // we can update content directly
            buffer_content.world = cgmath::Matrix4::from(rotation).into();
        }

        let image_num = swapchain.acquire_next_image(Duration::from_millis(1)).unwrap();
        submissions.push(vulkano::command_buffer::submit(&command_buffers[image_num], &queue).unwrap());
        swapchain.present(&queue, image_num).unwrap();

        for ev in window.window().poll_events() {
            match ev {
                winit::Event::Closed => return,
                _ => ()
            }
        }
    }
}

fn run() -> Result<()> {
    init_vulkan();
    Ok(())
}

fn main() {
    // Before we do anything. Initialize the logger.
    // This will only fail if we can't write to stderr.
    logger::init().expect("Could not initialize logger");

    // Run the program, and enter the block if we get an error.
    if let Err(ref e) = run() {
        error!("Program failed: {}", e);

        // Backtrace if we can. We may need RUST_BACKTRACE=1
        if let Some(backtrace) = e.backtrace() {
            debug!("{:?}", backtrace);
        }

        // Exit with error code 1
        ::std::process::exit(1);
    }
}

