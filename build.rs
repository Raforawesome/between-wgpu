use std::path::Path;
use std::process::Command;

fn main() {
    let src_dir = Path::new("src/shaders");

    for entry in src_dir.read_dir().expect("Failed to read src directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "slang")
            && path.file_stem().is_some_and(|n| n == "main_shader")
        {
            let output = src_dir.join("shader.wgsl");
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
