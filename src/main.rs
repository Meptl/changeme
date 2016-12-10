#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate glium;
extern crate time;

mod logger;

fn main() {
    logger::init().expect("Could not initialize logger");
    use glium::DisplayBuild;
    let display = match glium::glutin::WindowBuilder::new().build_glium() {
        Ok(d) => d,
        Err(e) => panic!(e)
    };
}
