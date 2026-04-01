use std::path::Path;
use std::process::Command;

fn main() {
    let src_dir = Path::new("src/shaders");

    for entry in src_dir.read_dir().expect("Failed to read src directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "slang") {
            let output = path.with_extension("wgsl");

            println!("cargo:rerun-if-changed={}", path.display());

            let status = Command::new("slangc")
                .arg(&path)
                .arg("-o")
                .arg(&output)
                .status()
                .expect("Failed to execute slangc");

            if !status.success() {
                panic!("Failed to compile shader: {:?}", path);
            }
        }
    }
}
