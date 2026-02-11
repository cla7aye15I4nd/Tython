use std::env;
use std::path::Path;
use std::process::Command;

const RUNTIME_SOURCES: &[&str] = &[
    "runtime/builtins.c",
    "runtime/str.c",
    "runtime/bytes.c",
    "runtime/bytearray.c",
];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Compile each .c file to a .o object
    let mut objects = Vec::new();
    for src in RUNTIME_SOURCES {
        let src_path = Path::new(src);
        let stem = src_path.file_stem().unwrap().to_str().unwrap();
        let obj = out_path.join(format!("{}.o", stem));

        let status = Command::new("clang")
            .arg("-c")
            .arg("-flto")
            .arg("-O2")
            .arg("-Iruntime")
            .arg("-o")
            .arg(&obj)
            .arg(src)
            .status()
            .unwrap_or_else(|_| panic!("Failed to compile {}", src));

        assert!(status.success(), "Failed to compile {}", src);
        objects.push(obj);
    }

    // Merge all bitcode objects into a single .o with llvm-link
    let runtime_obj = out_path.join("runtime.o");
    let mut cmd = Command::new("llvm-link");
    cmd.arg("-o").arg(&runtime_obj);
    for obj in &objects {
        cmd.arg(obj);
    }
    let status = cmd
        .status()
        .expect("Failed to link runtime objects with llvm-link");
    assert!(status.success(), "Failed to link runtime objects");

    println!("cargo:rustc-env=RUNTIME_BC_PATH={}", runtime_obj.display());
    for src in RUNTIME_SOURCES {
        println!("cargo:rerun-if-changed={}", src);
    }
    println!("cargo:rerun-if-changed=runtime/tython.h");
}
