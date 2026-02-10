use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let runtime_src = Path::new("runtime/runtime.c");
    let runtime_obj = Path::new(&out_dir).join("runtime.o");

    let status = Command::new("clang")
        .arg("-c")
        .arg("-flto")
        .arg("-O2")
        .arg("-o")
        .arg(&runtime_obj)
        .arg(runtime_src)
        .status()
        .expect("Failed to compile runtime.c to bitcode");

    assert!(status.success(), "Failed to compile runtime.c to bitcode");

    println!("cargo:rustc-env=RUNTIME_BC_PATH={}", runtime_obj.display());
    println!("cargo:rerun-if-changed=runtime/runtime.c");
}
