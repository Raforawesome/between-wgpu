use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let shader_dir = Path::new("src/shaders");
    let root_shader = shader_dir.join("shader.slang");
    let output =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set")).join("shader.wgsl");

    println!("cargo:rerun-if-env-changed=SLANGC");
    emit_shader_rerun_directives(shader_dir);

    let slangc = env::var_os("SLANGC").unwrap_or_else(|| OsString::from("slangc"));
    let output_status = Command::new(&slangc)
        .arg(&root_shader)
        .arg("-I")
        .arg(shader_dir)
        .arg("-target")
        .arg("wgsl")
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap_or_else(|err| panic!("Failed to execute {:?}: {err}", slangc));

    if !output_status.status.success() {
        panic!(
            "Failed to compile shader: {}\nstdout:\n{}\nstderr:\n{}",
            root_shader.display(),
            String::from_utf8_lossy(&output_status.stdout),
            String::from_utf8_lossy(&output_status.stderr),
        );
    }
}

fn emit_shader_rerun_directives(dir: &Path) {
    for entry in dir.read_dir().expect("Failed to read shader directory") {
        let path = entry.expect("Failed to read shader directory entry").path();

        if path.is_dir() {
            emit_shader_rerun_directives(&path);
        } else if path.extension().is_some_and(|ext| ext == "slang") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
