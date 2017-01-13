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
use render::Renderer;
use resource::ModelData;
use std::path::Path;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

mod core;
mod logger;
mod render;
mod resource;

static COLLADA_FILE: &'static str = "/home/yutoo/monkey.dae";

error_chain!{}

fn run() -> Result<()> {
    let mut renderer = render::Vulkan::new();

    /// Load vertex data
    let doc = match ColladaDocument::from_path(Path::new(COLLADA_FILE)) {
        Ok(file) => file,
        Err(e) => { println!("{}", e); return Ok(()) }
    };
    let vertex_buffer = CpuAccessibleBuffer::from_iter(&renderer.device, &BufferUsage::all(), Some(renderer.queue.family()),
                                                       doc.vertices().iter().cloned())
                                            .expect("failed to create vertex buffer");
    let normal_buffer = CpuAccessibleBuffer::from_iter(&renderer.device, &BufferUsage::all(), Some(renderer.queue.family()),
                                                       doc.normals().iter().cloned())
                                            .expect("failed to create normals buffer");
    let index_buffer = CpuAccessibleBuffer::from_iter(&renderer.device, &BufferUsage::all(), Some(renderer.queue.family()),
                                                      doc.indices().iter().cloned())
                                           .expect("failed to create index buffer");

    renderer.set_draw_buffers(vertex_buffer, normal_buffer, index_buffer);

    loop {
        renderer.render();
    }
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

