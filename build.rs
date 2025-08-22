use std::env;  
use copy_to_output::copy_to_output;  
  
fn main() {  
    // Copies assets/shaders/data files to output when changed
    println!("cargo:rerun-if-changed=shaders/compiled_shaders/*");
    println!("cargo:rerun-if-changed=assets/*");
    println!("cargo:rerun-if-changed=config.json");
    let profile = &env::var("PROFILE").unwrap();
    copy_to_output("shaders/compiled_shaders", profile).expect("Could not copy compiled shaders");  
    copy_to_output("assets", profile).expect("Could not copy assets");  
    copy_to_output("config.json", profile).expect("Could not copy config");  
}
