use std::env;
use std::path::{Path, PathBuf};
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
    "runtime/main.cpp",
];

const GC_SOURCE: &str = "runtime/gc/gc_boehm.cpp";

const RUNTIME_HEADERS: &[&str] = &[
    "runtime/tython.h",
    "runtime/gc/gc.h",
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

fn compile_runtime(out_path: &Path) -> PathBuf {
    let gc_define = "TYTHON_GC_BOEHM";

    let mut objects = Vec::new();

    // Compile runtime sources with GC flag
    for src in RUNTIME_SOURCES {
        let src_path = Path::new(src);
        let stem = src_path.file_stem().unwrap().to_str().unwrap();
        let obj = out_path.join(format!("{}_boehm.o", stem));

        let status = Command::new("clang++")
            .arg("-std=c++17")
            .arg("-c")
            .arg("-flto")
            .arg("-O2")
            .arg("-fexceptions")
            .arg(format!("-D{}", gc_define))
            .arg("-Iruntime")
            .arg("-o")
            .arg(&obj)
            .arg(src)
            .status()
            .unwrap_or_else(|_| panic!("Failed to compile {}", src));

        assert!(status.success(), "Failed to compile {}", src);
        objects.push(obj);
    }

    // Compile appropriate GC implementation
    let gc_obj = out_path.join("gc_boehm.o");
    let mut cmd = Command::new("clang++");
    cmd.arg("-std=c++17")
        .arg("-c")
        .arg("-flto")
        .arg("-O2")
        .arg("-fexceptions")
        .arg(format!("-D{}", gc_define))
        .arg("-Iruntime")
        .arg("-I/usr/include/gc")
        .arg("-o")
        .arg(&gc_obj)
        .arg(GC_SOURCE);

    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("Failed to compile {}", GC_SOURCE));
    assert!(status.success(), "Failed to compile {}", GC_SOURCE);
    objects.push(gc_obj);

    // Link all objects into runtime_boehm.o
    let runtime_obj = out_path.join("runtime_boehm.o");
    let mut cmd = Command::new("llvm-link");
    cmd.arg("-o").arg(&runtime_obj);
    for obj in &objects {
        cmd.arg(obj);
    }
    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("Failed to link {}", runtime_obj.display()));
    assert!(status.success(), "Failed to link {}", runtime_obj.display());

    runtime_obj
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Build Boehm runtime variant
    let runtime_boehm = compile_runtime(out_path);

    // Export path
    println!(
        "cargo:rustc-env=RUNTIME_BC_PATH_BOEHM={}",
        runtime_boehm.display()
    );

    // Default to Boehm
    println!(
        "cargo:rustc-env=RUNTIME_BC_PATH={}",
        runtime_boehm.display()
    );

    // Export stdlib directory path
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let stdlib_dir = Path::new(&manifest_dir).join("stdlib");
    println!("cargo:rustc-env=TYTHON_STDLIB_DIR={}", stdlib_dir.display());

    // Rerun if any source changes
    for src in RUNTIME_SOURCES {
        println!("cargo:rerun-if-changed={}", src);
    }
    println!("cargo:rerun-if-changed={}", GC_SOURCE);
    for header in RUNTIME_HEADERS {
        println!("cargo:rerun-if-changed={}", header);
    }
    println!("cargo:rerun-if-changed=stdlib");
}
