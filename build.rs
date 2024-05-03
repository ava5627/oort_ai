use std::path::Path;
extern crate rustsourcebundler;
use rustsourcebundler::Bundler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bundler: Bundler =
        Bundler::new(Path::new("src/bundle_input.rs"), Path::new("target/bundle_output.rs"));
    bundler.crate_name("oort_ai"); // again this must match the name in Cargo.toml
    bundler.run();
    Ok(())
}
