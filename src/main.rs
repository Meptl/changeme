#[macro_use]
extern crate glium;

fn main() {
    use glium::DisplayBuild;
    let display = match glium::glutin::WindowBuilder::new().build_glium() {
        Ok(d) => d,
        Err(e) => panic!(e)
    };
}
