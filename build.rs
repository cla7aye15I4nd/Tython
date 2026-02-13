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

const GC_SOURCES: &[&str] = &["runtime/gc/gc_naive.cpp", "runtime/gc/gc_boehm.cpp"];

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

fn compile_runtime(out_path: &Path, gc_type: &str) -> PathBuf {
    let gc_define = match gc_type {
        "naive" => "TYTHON_GC_NAIVE",
        "boehm" => "TYTHON_GC_BOEHM",
        _ => panic!("Unknown GC type: {}", gc_type),
    };

    let mut objects = Vec::new();

    // Compile runtime sources with GC flag
    for src in RUNTIME_SOURCES {
        let src_path = Path::new(src);
        let stem = src_path.file_stem().unwrap().to_str().unwrap();
        let obj = out_path.join(format!("{}_{}.o", stem, gc_type));

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
    let gc_src = match gc_type {
        "naive" => "runtime/gc/gc_naive.cpp",
        "boehm" => "runtime/gc/gc_boehm.cpp",
        _ => unreachable!(),
    };

    let gc_obj = out_path.join(format!("gc_{}.o", gc_type));
    let mut cmd = Command::new("clang++");
    cmd.arg("-std=c++17")
        .arg("-c")
        .arg("-flto")
        .arg("-O2")
        .arg("-fexceptions")
        .arg(format!("-D{}", gc_define))
        .arg("-Iruntime");

    // Add Boehm GC include path if needed
    if gc_type == "boehm" {
        cmd.arg("-I/usr/include/gc");
    }

    cmd.arg("-o").arg(&gc_obj).arg(gc_src);

    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("Failed to compile {}", gc_src));
    assert!(status.success(), "Failed to compile {}", gc_src);
    objects.push(gc_obj);

    // Link all objects into runtime_{gc_type}.o
    let runtime_obj = out_path.join(format!("runtime_{}.o", gc_type));
    let mut cmd = Command::new("llvm-link");
    cmd.arg("-o").arg(&runtime_obj);
    for obj in &objects {
        cmd.arg(obj);
    }
    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("Failed to link runtime_{}.o", gc_type));
    assert!(status.success(), "Failed to link runtime_{}.o", gc_type);

    runtime_obj
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Build both runtime variants
    let runtime_naive = compile_runtime(out_path, "naive");
    let runtime_boehm = compile_runtime(out_path, "boehm");

    // Export both paths
    println!(
        "cargo:rustc-env=RUNTIME_BC_PATH_NAIVE={}",
        runtime_naive.display()
    );
    println!(
        "cargo:rustc-env=RUNTIME_BC_PATH_BOEHM={}",
        runtime_boehm.display()
    );

    // Default to Boehm
    println!(
        "cargo:rustc-env=RUNTIME_BC_PATH={}",
        runtime_boehm.display()
    );

    // Rerun if any source changes
    for src in RUNTIME_SOURCES {
        println!("cargo:rerun-if-changed={}", src);
    }
    for src in GC_SOURCES {
        println!("cargo:rerun-if-changed={}", src);
    }
    for header in RUNTIME_HEADERS {
        println!("cargo:rerun-if-changed={}", header);
    }
}
