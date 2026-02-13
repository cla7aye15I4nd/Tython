use std::env;
use std::path::Path;
use std::process::Command;

const RUNTIME_SOURCES: &[&str] = &[
    "runtime/builtins/print.cpp",
    "runtime/builtins/math.cpp",
    "runtime/builtins/core.cpp",
    "runtime/str/str.cpp",
    "runtime/bytes/bytes.cpp",
    "runtime/bytearray/bytearray.cpp",
    "runtime/list/list.cpp",
    "runtime/dict/dict.cpp",
    "runtime/set/set.cpp",
    "runtime/exception.cpp",
];
const RUNTIME_HEADERS: &[&str] = &[
    "runtime/tython.h",
    "runtime/builtins/builtins.h",
    "runtime/builtins/common.h",
    "runtime/builtins/print.h",
    "runtime/builtins/math.h",
    "runtime/builtins/core.h",
    "runtime/str/str.h",
    "runtime/bytes/bytes.h",
    "runtime/bytearray/bytearray.h",
    "runtime/list/list.h",
    "runtime/dict/dict.h",
    "runtime/set/set.h",
    "runtime/internal/vec.h",
    "runtime/internal/buf.h",
];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Compile each .cpp file to a .o bitcode object
    let mut objects = Vec::new();
    for src in RUNTIME_SOURCES {
        let src_path = Path::new(src);
        let stem = src_path.file_stem().unwrap().to_str().unwrap();
        let obj = out_path.join(format!("{}.o", stem));

        let status = Command::new("clang++")
            .arg("-std=c++17")
            .arg("-c")
            .arg("-flto")
            .arg("-O2")
            .arg("-fexceptions")
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
    for header in RUNTIME_HEADERS {
        println!("cargo:rerun-if-changed={}", header);
    }
}
