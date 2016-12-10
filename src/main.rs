#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate glium;
extern crate time;

mod logger;

error_chain!{}

fn run() -> Result<()> {
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

